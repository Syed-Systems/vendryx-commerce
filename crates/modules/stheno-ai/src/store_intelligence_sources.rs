//! Source-aware metric contracts for Store Intelligence adapters (ADR-035).

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::diesel::sql_types::{BigInt, Date, Nullable, Numeric, Text, Timestamptz};
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{Conn, Error, Result as DbResult};

/// Coverage reported by an adapter for one source and observation window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricCoverage {
    Complete,
    Partial,
    Unavailable,
}

/// A raw, tenant-scoped metric with its provider meaning and collection window.
///
/// `definition` identifies the provider's metric semantics and version. Values
/// with different definitions or units are never compared or combined.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceMetricObservation {
    pub tenant_id: String,
    pub source: String,
    pub account_id: String,
    pub scope: String,
    pub metric: String,
    pub definition: String,
    pub unit: String,
    pub value: Decimal,
    pub denominator: Option<Decimal>,
    pub sample_size: Option<u64>,
    pub window_started_on: NaiveDate,
    pub window_ended_on: NaiveDate,
    pub window_timezone: String,
    pub ingested_at: DateTime<Utc>,
    pub coverage: MetricCoverage,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreIntelligenceSourceSnapshot {
    pub source: String,
    pub account_id: String,
    pub scope: String,
    pub metric: String,
    pub definition: String,
    pub unit: String,
    pub value: Decimal,
    pub denominator: Option<Decimal>,
    pub sample_size: Option<i64>,
    pub window_started_on: NaiveDate,
    pub window_ended_on: NaiveDate,
    pub window_timezone: String,
    pub ingested_at: DateTime<Utc>,
    pub coverage: MetricCoverage,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreIntelligenceSourceChange {
    pub source: String,
    pub account_id: String,
    pub scope: String,
    pub metric: String,
    pub definition: String,
    pub unit: String,
    pub previous_value: Decimal,
    pub current_value: Decimal,
    pub change_basis_points: Decimal,
    pub previous_window_started_on: NaiveDate,
    pub previous_window_ended_on: NaiveDate,
    pub current_window_started_on: NaiveDate,
    pub current_window_ended_on: NaiveDate,
    pub window_timezone: String,
}

/// An exact aggregate across source observations with explicitly disjoint scopes.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompatibleSourceMetricAggregate {
    pub tenant_id: String,
    pub sources: Vec<String>,
    pub metric: String,
    pub definition: String,
    pub unit: String,
    pub value: Decimal,
    pub sample_size: Option<u64>,
    pub window_started_on: NaiveDate,
    pub window_ended_on: NaiveDate,
    pub window_timezone: String,
}

/// Sums only complete additive count observations whose normalized meaning and
/// windows match and whose scopes identify disjoint partitions.
///
/// `scope` is the explicit partition identity for this operation. A metric
/// provider must not call this for overlapping account or content populations.
pub fn aggregate_compatible_source_observations(
    tenant_id: &str,
    observations: &[SourceMetricObservation],
) -> Option<CompatibleSourceMetricAggregate> {
    if observations.len() < 2 || observations.len() > 32 {
        return None;
    }
    let first = observations.first()?;
    if first.unit != "count"
        || first.denominator.is_some()
        || first.coverage != MetricCoverage::Complete
        || first.tenant_id != tenant_id
        || first.validate().is_err()
    {
        return None;
    }

    let mut sources = std::collections::BTreeSet::new();
    let mut scopes = std::collections::HashSet::new();
    let mut source_accounts = std::collections::HashSet::new();
    let mut value = Decimal::ZERO;
    let mut sample_size = Some(0_u64);
    for observation in observations {
        if observation.tenant_id != tenant_id
            || observation.metric != first.metric
            || observation.definition != first.definition
            || observation.unit != first.unit
            || observation.denominator.is_some()
            || observation.window_started_on != first.window_started_on
            || observation.window_ended_on != first.window_ended_on
            || observation.window_timezone != first.window_timezone
            || observation.coverage != MetricCoverage::Complete
            || observation.validate().is_err()
            || !scopes.insert(observation.scope.as_str())
            || !source_accounts
                .insert((observation.source.as_str(), observation.account_id.as_str()))
        {
            return None;
        }
        sources.insert(observation.source.clone());
        value = value.checked_add(observation.value)?;
        sample_size = match (sample_size, observation.sample_size) {
            (Some(total), Some(count)) => Some(total.checked_add(count)?),
            _ => None,
        };
    }
    if sources.len() < 2 {
        return None;
    }

    Some(CompatibleSourceMetricAggregate {
        tenant_id: tenant_id.to_string(),
        sources: sources.into_iter().collect(),
        metric: first.metric.clone(),
        definition: first.definition.clone(),
        unit: first.unit.clone(),
        value,
        sample_size,
        window_started_on: first.window_started_on,
        window_ended_on: first.window_ended_on,
        window_timezone: first.window_timezone.clone(),
    })
}

/// Converts retained observations into adjacent-window changes without exposing tenant IDs.
pub fn compare_adjacent_source_snapshots(
    tenant_id: &str,
    snapshots: &[StoreIntelligenceSourceSnapshot],
) -> Vec<StoreIntelligenceSourceChange> {
    let observations = snapshots
        .iter()
        .filter_map(|snapshot| {
            Some(SourceMetricObservation {
                tenant_id: tenant_id.to_string(),
                source: snapshot.source.clone(),
                account_id: snapshot.account_id.clone(),
                scope: snapshot.scope.clone(),
                metric: snapshot.metric.clone(),
                definition: snapshot.definition.clone(),
                unit: snapshot.unit.clone(),
                value: snapshot.value,
                denominator: snapshot.denominator,
                sample_size: snapshot.sample_size.map(u64::try_from).transpose().ok()?,
                window_started_on: snapshot.window_started_on,
                window_ended_on: snapshot.window_ended_on,
                window_timezone: snapshot.window_timezone.clone(),
                ingested_at: snapshot.ingested_at,
                coverage: snapshot.coverage,
            })
        })
        .collect::<Vec<_>>();
    compare_adjacent_source_observations(tenant_id, &observations)
}

/// Compares only immediately adjacent complete windows with matching source semantics.
pub fn compare_adjacent_source_observations(
    tenant_id: &str,
    observations: &[SourceMetricObservation],
) -> Vec<StoreIntelligenceSourceChange> {
    let mut changes = Vec::new();
    for current in observations {
        let Some(previous) = observations
            .iter()
            .filter(|candidate| {
                candidate.metric == current.metric
                    && candidate.window_ended_on == current.window_started_on
                    && current.is_comparable_to(candidate)
            })
            .max_by_key(|candidate| candidate.ingested_at)
        else {
            continue;
        };
        let Some(change_basis_points) = current.relative_change_basis_points(previous) else {
            continue;
        };
        if current.tenant_id != tenant_id {
            continue;
        }
        changes.push(StoreIntelligenceSourceChange {
            source: current.source.clone(),
            account_id: current.account_id.clone(),
            scope: current.scope.clone(),
            metric: current.metric.clone(),
            definition: current.definition.clone(),
            unit: current.unit.clone(),
            previous_value: previous.value,
            current_value: current.value,
            change_basis_points,
            previous_window_started_on: previous.window_started_on,
            previous_window_ended_on: previous.window_ended_on,
            current_window_started_on: current.window_started_on,
            current_window_ended_on: current.window_ended_on,
            window_timezone: current.window_timezone.clone(),
        });
    }
    changes.sort_by(|left, right| left.metric.cmp(&right.metric));
    changes
}

#[derive(QueryableByName)]
struct SnapshotRow {
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    scope: String,
    #[diesel(sql_type = Text)]
    metric: String,
    #[diesel(sql_type = Text)]
    definition: String,
    #[diesel(sql_type = Text)]
    unit: String,
    #[diesel(sql_type = Numeric)]
    value: Decimal,
    #[diesel(sql_type = Nullable<Numeric>)]
    denominator: Option<Decimal>,
    #[diesel(sql_type = Nullable<BigInt>)]
    sample_size: Option<i64>,
    #[diesel(sql_type = Date)]
    window_started_on: NaiveDate,
    #[diesel(sql_type = Date)]
    window_ended_on: NaiveDate,
    #[diesel(sql_type = Text)]
    window_timezone: String,
    #[diesel(sql_type = Timestamptz)]
    ingested_at: DateTime<Utc>,
    #[diesel(sql_type = Text)]
    coverage: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StoreIntelligenceSourceService;

impl StoreIntelligenceSourceService {
    /// Records a bounded observation window and prunes data beyond the retention period atomically.
    pub async fn record_snapshot(
        &self,
        conn: &mut Conn<'_>,
        source: &str,
        account_id: &str,
        observations: &[SourceMetricObservation],
    ) -> DbResult<()> {
        let tenant_id = stheno_db::tenant::current()?;
        if source.is_empty()
            || source.len() > 64
            || account_id.is_empty()
            || account_id.len() > 512
            || observations.len() > 100
        {
            return Err(source_input_error("invalid source snapshot identity"));
        }
        let source = source.to_string();
        let account_id = account_id.to_string();
        let mut metrics = std::collections::HashSet::new();
        for item in observations {
            if item.validate().is_err()
                || item.tenant_id != tenant_id
                || item.source != source
                || item.account_id != account_id
                || !metrics.insert((
                    item.metric.as_str(),
                    item.window_started_on,
                    item.window_ended_on,
                ))
            {
                return Err(source_input_error("invalid source observation snapshot"));
            }
        }
        let items = observations.to_vec();
        stheno_db::transaction::with_transaction(conn, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "DELETE FROM ai.store_intelligence_source_observation \
                     WHERE ingested_at < now() - INTERVAL '400 days'",
                )
                .execute(conn)
                .await
                .map_err(database_error)?;

                for item in items {
                    let coverage = match item.coverage {
                        MetricCoverage::Complete => "complete",
                        MetricCoverage::Partial => "partial",
                        MetricCoverage::Unavailable => "unavailable",
                    };
                    let sample_size = item
                        .sample_size
                        .map(i64::try_from)
                        .transpose()
                        .map_err(|_| source_input_error("sample size exceeds database range"))?;
                    diesel::sql_query(
                        "INSERT INTO ai.store_intelligence_source_observation \
                         (tenant_id, source, account_id, scope, metric, definition, unit, value, denominator, sample_size, \
                          window_started_on, window_ended_on, window_timezone, ingested_at, coverage) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
                         ON CONFLICT (tenant_id, source, account_id, metric, window_started_on, window_ended_on) \
                         DO UPDATE SET scope = EXCLUDED.scope, definition = EXCLUDED.definition, unit = EXCLUDED.unit, \
                            value = EXCLUDED.value, denominator = EXCLUDED.denominator, sample_size = EXCLUDED.sample_size, \
                            window_timezone = EXCLUDED.window_timezone, ingested_at = EXCLUDED.ingested_at, coverage = EXCLUDED.coverage",
                    )
                    .bind::<Text, _>(&tenant_id)
                    .bind::<Text, _>(&item.source)
                    .bind::<Text, _>(&item.account_id)
                    .bind::<Text, _>(&item.scope)
                    .bind::<Text, _>(&item.metric)
                    .bind::<Text, _>(&item.definition)
                    .bind::<Text, _>(&item.unit)
                    .bind::<Numeric, _>(item.value)
                    .bind::<Nullable<Numeric>, _>(item.denominator)
                    .bind::<Nullable<BigInt>, _>(sample_size)
                    .bind::<Date, _>(item.window_started_on)
                    .bind::<Date, _>(item.window_ended_on)
                    .bind::<Text, _>(&item.window_timezone)
                    .bind::<Timestamptz, _>(item.ingested_at)
                    .bind::<Text, _>(coverage)
                    .execute(conn)
                    .await
                    .map_err(database_error)?;
                }
                Ok(())
            })
        })
        .await
    }

    /// Reads the most recent bounded observation windows for one source/account.
    pub async fn list_recent(
        &self,
        conn: &mut Conn<'_>,
        source: &str,
        account_id: &str,
    ) -> DbResult<Vec<StoreIntelligenceSourceSnapshot>> {
        if source.is_empty() || source.len() > 64 || account_id.is_empty() || account_id.len() > 512
        {
            return Err(source_input_error("invalid source snapshot lookup"));
        }
        diesel::sql_query(
            "SELECT source, account_id, scope, metric, definition, unit, value, denominator, sample_size, \
             window_started_on, window_ended_on, window_timezone, ingested_at, coverage \
             FROM ai.store_intelligence_source_observation \
             WHERE source = $1 AND account_id = $2 \
               AND ingested_at >= now() - INTERVAL '400 days' \
             ORDER BY ingested_at DESC, metric LIMIT 100",
        )
        .bind::<Text, _>(source)
        .bind::<Text, _>(account_id)
        .load::<SnapshotRow>(conn)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(database_error)
    }
}

impl From<SnapshotRow> for StoreIntelligenceSourceSnapshot {
    fn from(row: SnapshotRow) -> Self {
        Self {
            source: row.source,
            account_id: row.account_id,
            scope: row.scope,
            metric: row.metric,
            definition: row.definition,
            unit: row.unit,
            value: row.value,
            denominator: row.denominator,
            sample_size: row.sample_size,
            window_started_on: row.window_started_on,
            window_ended_on: row.window_ended_on,
            window_timezone: row.window_timezone,
            ingested_at: row.ingested_at,
            coverage: match row.coverage.as_str() {
                "complete" => MetricCoverage::Complete,
                "partial" => MetricCoverage::Partial,
                _ => MetricCoverage::Unavailable,
            },
        }
    }
}

fn database_error(error: diesel::result::Error) -> Error {
    Error::Database(error.to_string())
}

fn source_input_error(message: &str) -> Error {
    Error::Invalid {
        code: "invalid_source_observation",
        message: message.to_string(),
        field: None,
    }
}

impl SourceMetricObservation {
    /// Rejects malformed or semantically unusable provider observations.
    pub fn validate(&self) -> std::result::Result<(), &'static str> {
        for (name, value, max_len) in [
            ("tenant_id", self.tenant_id.as_str(), 128),
            ("source", self.source.as_str(), 64),
            ("account_id", self.account_id.as_str(), 256),
            ("scope", self.scope.as_str(), 128),
            ("metric", self.metric.as_str(), 128),
            ("definition", self.definition.as_str(), 128),
            ("unit", self.unit.as_str(), 64),
            ("window_timezone", self.window_timezone.as_str(), 64),
        ] {
            if value.trim().is_empty() || value.len() > max_len {
                return Err(match name {
                    "tenant_id" => "tenant_id_invalid",
                    "source" => "source_invalid",
                    "account_id" => "account_id_invalid",
                    "scope" => "scope_invalid",
                    "metric" => "metric_invalid",
                    "definition" => "definition_invalid",
                    "unit" => "unit_invalid",
                    _ => "window_timezone_invalid",
                });
            }
        }
        if self.window_started_on >= self.window_ended_on {
            return Err("window_invalid");
        }
        if self.value < Decimal::ZERO || self.denominator.is_some_and(|v| v < Decimal::ZERO) {
            return Err("negative_metric_value");
        }
        if self.sample_size == Some(0) {
            return Err("sample_size_invalid");
        }
        if self.coverage == MetricCoverage::Complete && self.denominator == Some(Decimal::ZERO) {
            return Err("zero_denominator");
        }
        Ok(())
    }

    /// True only for complete, same-meaning, tenant/account-scoped adjacent windows.
    pub fn is_comparable_to(&self, previous: &Self) -> bool {
        self.coverage == MetricCoverage::Complete
            && previous.coverage == MetricCoverage::Complete
            && self.validate().is_ok()
            && previous.validate().is_ok()
            && self.tenant_id == previous.tenant_id
            && self.source == previous.source
            && self.account_id == previous.account_id
            && self.scope == previous.scope
            && self.metric == previous.metric
            && self.definition == previous.definition
            && self.unit == previous.unit
            && self.denominator.is_some() == previous.denominator.is_some()
            && self.sample_size.is_some() == previous.sample_size.is_some()
            && self.window_timezone == previous.window_timezone
            && self.window_ended_on - self.window_started_on
                == previous.window_ended_on - previous.window_started_on
            && self.window_started_on == previous.window_ended_on
    }

    /// Returns exact basis-point change only when the earlier value is positive.
    pub fn relative_change_basis_points(&self, previous: &Self) -> Option<Decimal> {
        if !self.is_comparable_to(previous) || previous.value <= Decimal::ZERO {
            return None;
        }
        Some(((self.value - previous.value) * Decimal::from(10_000) / previous.value).normalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn observation(start: NaiveDate) -> SourceMetricObservation {
        SourceMetricObservation {
            tenant_id: "tenant-a".to_string(),
            source: "search_console".to_string(),
            account_id: "property-a".to_string(),
            scope: "merchant_property".to_string(),
            metric: "clicks".to_string(),
            definition: "search-console-clicks-v1".to_string(),
            unit: "count".to_string(),
            value: Decimal::from(12),
            denominator: None,
            sample_size: Some(20),
            window_started_on: start,
            window_ended_on: start + chrono::Days::new(7),
            window_timezone: "UTC".to_string(),
            ingested_at: DateTime::<Utc>::from_timestamp(1_798_804_800, 0).expect("timestamp"),
            coverage: MetricCoverage::Complete,
        }
    }

    #[test]
    fn only_complete_adjacent_windows_with_matching_source_semantics_compare() {
        let previous_start = NaiveDate::from_ymd_opt(2026, 10, 1).expect("valid date");
        let previous = observation(previous_start);
        let current = observation(previous.window_ended_on);
        assert!(current.is_comparable_to(&previous));

        let mut other_tenant = current.clone();
        other_tenant.tenant_id = "tenant-b".to_string();
        assert!(!other_tenant.is_comparable_to(&previous));

        let mut other_definition = current.clone();
        other_definition.definition = "search-console-impressions-v1".to_string();
        assert!(!other_definition.is_comparable_to(&previous));

        let mut other_timezone = current.clone();
        other_timezone.window_timezone = "America/Los_Angeles".to_string();
        assert!(!other_timezone.is_comparable_to(&previous));

        let mut partial = current.clone();
        partial.coverage = MetricCoverage::Partial;
        assert!(!partial.is_comparable_to(&previous));

        let mut gapped = current;
        gapped.window_started_on += chrono::TimeDelta::days(1);
        assert!(!gapped.is_comparable_to(&previous));
    }

    #[test]
    fn validation_rejects_zero_denominators_and_invalid_collection_times() {
        let start = NaiveDate::from_ymd_opt(2026, 10, 1).expect("valid date");
        let mut item = observation(start);
        item.denominator = Some(Decimal::ZERO);
        assert_eq!(item.validate(), Err("zero_denominator"));

        item.denominator = None;
        item.window_ended_on = item.window_started_on;
        assert_eq!(item.validate(), Err("window_invalid"));

        item.window_ended_on += chrono::TimeDelta::days(7);
        item.value = Decimal::from(-1);
        assert_eq!(item.validate(), Err("negative_metric_value"));
    }

    #[test]
    fn relative_change_requires_compatible_windows_and_positive_baseline() {
        let start = NaiveDate::from_ymd_opt(2026, 10, 1).expect("valid date");
        let previous = observation(start);
        let mut current = observation(previous.window_ended_on);
        current.value = Decimal::from(15);
        assert_eq!(
            current.relative_change_basis_points(&previous),
            Some(Decimal::from(2_500))
        );

        let mut zero_baseline = previous.clone();
        zero_baseline.value = Decimal::ZERO;
        assert_eq!(current.relative_change_basis_points(&zero_baseline), None);

        current.coverage = MetricCoverage::Partial;
        assert_eq!(current.relative_change_basis_points(&previous), None);
    }

    #[test]
    fn source_change_report_uses_only_adjacent_complete_compatible_windows() {
        let start = NaiveDate::from_ymd_opt(2026, 10, 1).expect("valid date");
        let previous = observation(start);
        let mut current = observation(previous.window_ended_on);
        current.value = Decimal::from(15);
        let mut later = current.clone();
        later.window_started_on = current.window_ended_on;
        later.window_ended_on += chrono::TimeDelta::days(7);
        later.value = Decimal::from(18);

        let changes = compare_adjacent_source_observations(
            "tenant-a",
            &[previous.clone(), current.clone(), later],
        );
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].change_basis_points, Decimal::from(2_500));
        assert_eq!(changes[1].change_basis_points, Decimal::from(2_000));

        let mut partial = current;
        partial.coverage = MetricCoverage::Partial;
        let changes = compare_adjacent_source_observations("tenant-a", &[previous, partial]);
        assert!(changes.is_empty());

        let old = observation(start);
        let mut new = observation(old.window_ended_on);
        new.value = Decimal::from(15);
        let snapshots = [old, new]
            .iter()
            .map(|item| StoreIntelligenceSourceSnapshot {
                source: item.source.clone(),
                account_id: item.account_id.clone(),
                scope: item.scope.clone(),
                metric: item.metric.clone(),
                definition: item.definition.clone(),
                unit: item.unit.clone(),
                value: item.value,
                denominator: item.denominator,
                sample_size: item
                    .sample_size
                    .map(|sample| i64::try_from(sample).expect("small sample size")),
                window_started_on: item.window_started_on,
                window_ended_on: item.window_ended_on,
                window_timezone: item.window_timezone.clone(),
                ingested_at: item.ingested_at,
                coverage: item.coverage,
            })
            .collect::<Vec<_>>();
        let changes = compare_adjacent_source_snapshots("tenant-a", &snapshots);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_basis_points, Decimal::from(2_500));
    }

    #[test]
    fn cross_source_aggregation_requires_compatible_disjoint_complete_counts() {
        let mut facebook = observation(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap());
        facebook.source = "facebook".to_string();
        facebook.account_id = "page-a".to_string();
        facebook.scope = "region:north".to_string();
        facebook.definition = "retail-engagement-count-v1".to_string();
        facebook.value = Decimal::from(7);
        let mut instagram = facebook.clone();
        instagram.source = "instagram".to_string();
        instagram.account_id = "account-b".to_string();
        instagram.scope = "region:south".to_string();
        instagram.value = Decimal::from(5);

        let aggregate = aggregate_compatible_source_observations(
            "tenant-a",
            &[facebook.clone(), instagram.clone()],
        )
        .expect("compatible disjoint count partitions should aggregate");
        assert_eq!(aggregate.value, Decimal::from(12));
        assert_eq!(aggregate.sources, ["facebook", "instagram"]);
        assert_eq!(aggregate.sample_size, Some(40));

        let mut overlapping = instagram.clone();
        overlapping.scope = facebook.scope.clone();
        assert!(aggregate_compatible_source_observations(
            "tenant-a",
            &[facebook.clone(), overlapping]
        )
        .is_none());

        let mut mismatched = instagram.clone();
        mismatched.definition = "unrelated-provider-count-v1".to_string();
        assert!(aggregate_compatible_source_observations(
            "tenant-a",
            &[facebook.clone(), mismatched]
        )
        .is_none());

        let mut partial = instagram;
        partial.coverage = MetricCoverage::Partial;
        assert!(
            aggregate_compatible_source_observations("tenant-a", &[facebook, partial]).is_none()
        );
    }
}
