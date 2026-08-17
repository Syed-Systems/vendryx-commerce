//! P11-17: `region` on Postgres.
//!
//! The lifecycle assertions are unchanged from the in-memory version; what is
//! new is that a second `RegionService` built over the same database sees the
//! first one's writes. That is the property `RwLock<HashMap>` could never have.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_region::{CreateRegionInput, RegionService, UpdateRegionInput};

async fn service() -> (TestDb, RegionService) {
    let db = TestDb::migrated().await;
    let svc = RegionService::new(create_pool(&db.url, 5).expect("pool"));
    (db, svc)
}

#[tokio::test]
async fn region_and_country_lifecycle() {
    let (db, service) = service().await;

    let eu_region = service
        .create_region(CreateRegionInput {
            name: "European Union".to_string(),
            currency_code: "eur".to_string(),
            automatic_taxes: Some(true),
            countries: vec!["de".to_string(), "fr".to_string()],
        })
        .await
        .expect("must create EU region");

    assert_eq!(eu_region.name, "European Union");
    assert_eq!(eu_region.currency_code, "eur");
    assert_eq!(eu_region.countries.len(), 2);

    let region_id = eu_region.id.to_string();
    assert!(
        region_id.starts_with("reg_"),
        "the region id lost its prefix: {region_id}"
    );

    let retrieved = service
        .get_region(&region_id)
        .await
        .expect("must get region");
    assert_eq!(retrieved.name, "European Union");
    assert_eq!(retrieved.countries.len(), 2);

    let de = service
        .get_country_by_iso2("de")
        .await
        .expect("must get DE");
    assert_eq!(de.iso_2, "de");

    let list = service.list_regions().await.expect("must list regions");
    assert_eq!(list.len(), 1);

    db.drop().await;
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    // The single assertion Phase 1 could not make.
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let region_id = {
        let first = RegionService::new(pool.clone());
        first
            .create_region(CreateRegionInput {
                name: "Persistent".to_string(),
                currency_code: "usd".to_string(),
                automatic_taxes: None,
                countries: vec!["us".to_string()],
            })
            .await
            .expect("create")
            .id
            .to_string()
    };

    // A brand new service, sharing nothing with the first but the database.
    let second = RegionService::new(pool);
    let found = second
        .get_region(&region_id)
        .await
        .expect("the region must survive the service that created it");
    assert_eq!(found.name, "Persistent");
    assert_eq!(found.countries.len(), 1);

    db.drop().await;
}

#[tokio::test]
async fn retrieving_an_unknown_region_is_not_found() {
    let (db, service) = service().await;
    let err = service
        .get_region("reg_01ARZ3NDEKTSV4RRFFQ69G5FAV")
        .await
        .expect_err("must not invent a region");
    assert!(err.to_string().contains("reg_"), "got: {err}");
    db.drop().await;
}

#[tokio::test]
async fn a_malformed_region_id_is_rejected() {
    let (db, service) = service().await;
    assert!(
        service.get_region("not-an-id").await.is_err(),
        "a malformed id must not match anything"
    );
    db.drop().await;
}

#[tokio::test]
async fn update_replaces_fields_and_countries() {
    let (db, service) = service().await;

    let created = service
        .create_region(CreateRegionInput {
            name: "Old".to_string(),
            currency_code: "usd".to_string(),
            automatic_taxes: Some(false),
            countries: vec!["us".to_string()],
        })
        .await
        .expect("create");
    let id = created.id.to_string();

    let updated = service
        .update_region(
            &id,
            UpdateRegionInput {
                name: Some("New".to_string()),
                currency_code: Some("EUR".to_string()),
                automatic_taxes: Some(true),
                countries: Some(vec!["de".to_string(), "at".to_string()]),
            },
        )
        .await
        .expect("update");

    assert_eq!(updated.name, "New");
    assert_eq!(updated.currency_code, "eur", "currency must be normalised");
    assert!(updated.automatic_taxes);
    assert_eq!(updated.countries.len(), 2);

    // And it persisted, rather than only being returned.
    let refetched = service.get_region(&id).await.expect("refetch");
    assert_eq!(refetched.name, "New");
    assert_eq!(refetched.countries.len(), 2);

    db.drop().await;
}

#[tokio::test]
async fn a_deleted_region_disappears_from_reads_and_lists() {
    let (db, service) = service().await;

    let created = service
        .create_region(CreateRegionInput {
            name: "Doomed".to_string(),
            currency_code: "usd".to_string(),
            automatic_taxes: None,
            countries: vec![],
        })
        .await
        .expect("create");
    let id = created.id.to_string();

    service.delete_region(&id).await.expect("delete");

    assert!(
        service.get_region(&id).await.is_err(),
        "a deleted region must not be retrievable"
    );
    assert!(
        service.list_regions().await.expect("list").is_empty(),
        "a deleted region must not be listed"
    );

    // Soft, not hard: the row is still on disk for audit and erasure replay.
    let rows = db.count("SELECT count(*) FROM region.region").await;
    assert_eq!(rows, 1, "delete_region must be a soft delete");

    db.drop().await;
}

#[tokio::test]
async fn a_region_with_no_countries_is_valid() {
    let (db, service) = service().await;
    let created = service
        .create_region(CreateRegionInput {
            name: "Empty".to_string(),
            currency_code: "usd".to_string(),
            automatic_taxes: None,
            countries: vec![],
        })
        .await
        .expect("create");
    assert!(created.countries.is_empty());

    let refetched = service
        .get_region(&created.id.to_string())
        .await
        .expect("refetch");
    assert!(refetched.countries.is_empty());

    db.drop().await;
}

#[tokio::test]
async fn no_in_memory_state_remains_in_the_service() {
    // Guards the P11 definition of done: a migrated module keeps no state of
    // its own, so two handles are interchangeable.
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let a = RegionService::new(pool.clone());
    let b = RegionService::new(pool);

    a.create_region(CreateRegionInput {
        name: "Shared".to_string(),
        currency_code: "gbp".to_string(),
        automatic_taxes: None,
        countries: vec!["gb".to_string()],
    })
    .await
    .expect("create through A");

    let seen_by_b = b.list_regions().await.expect("list through B");
    assert_eq!(seen_by_b.len(), 1, "B did not see A's write");

    db.drop().await;
}
