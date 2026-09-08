//! Courier status vocabulary (DATA_MODEL §2.6, PHASE3_TASKS P23-01, P23-11).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Unified engine-side courier status vocabulary.
///
/// Every courier status maps into one of these canonical variants, or falls into
/// `Unknown(String)` which logs and raises an alert rather than silently ignoring it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CourierStatus {
    Created,
    PickedUp,
    InTransit,
    OutForDelivery,
    Delivered,
    PartiallyDelivered,
    FailedAttempt,
    OnHold,
    ReturnInTransit,
    Returned,
    Cancelled,
    Lost,
    Damaged,
    Unknown(String),
}

impl CourierStatus {
    /// Parse a status string from a courier into a canonical `CourierStatus`.
    ///
    /// If unrecognized, logs a warning/alert and returns `CourierStatus::Unknown(raw)`.
    pub fn parse(raw: &str) -> Self {
        let normalized = raw.trim().to_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "created" | "pending" | "order_placed" | "processing" | "in_review" => Self::Created,
            "picked_up" | "pickup" | "picked" | "collected" => Self::PickedUp,
            "in_transit" | "transit" | "dispatched" | "shipped" | "received_by_hub" => {
                Self::InTransit
            }
            "out_for_delivery" | "delivery_in_progress" => Self::OutForDelivery,
            "delivered" | "completed" | "successful" => Self::Delivered,
            "partially_delivered" | "partial_delivery" => Self::PartiallyDelivered,
            "failed_attempt" | "delivery_failed" | "attempted" | "customer_unavailable" => {
                Self::FailedAttempt
            }
            "on_hold" | "hold" | "delayed" => Self::OnHold,
            "return_in_transit" | "returning" | "return_in_progress" => Self::ReturnInTransit,
            "returned" | "return" | "returned_to_merchant" => Self::Returned,
            "cancelled" | "canceled" | "rejected" => Self::Cancelled,
            "lost" => Self::Lost,
            "damaged" => Self::Damaged,
            _ => {
                tracing::warn!(
                    raw_status = %raw,
                    "Unknown courier status encountered: '{}' - alert raised, preserving in Unknown variant",
                    raw
                );
                Self::Unknown(raw.to_string())
            }
        }
    }

    /// True if the parcel has been delivered to the recipient.
    pub fn is_delivered(&self) -> bool {
        matches!(self, Self::Delivered)
    }

    /// True if this is a terminal state (no further updates expected).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Delivered
                | Self::PartiallyDelivered
                | Self::Returned
                | Self::Cancelled
                | Self::Lost
                | Self::Damaged
        )
    }

    /// True if this indicates a return flow.
    pub fn is_return(&self) -> bool {
        matches!(self, Self::ReturnInTransit | Self::Returned)
    }

    /// True if the status was not recognized in the canonical vocabulary.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    /// Returns the raw status string if this is an Unknown variant.
    pub fn unknown_raw(&self) -> Option<&str> {
        match self {
            Self::Unknown(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Canonical string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::PickedUp => "picked_up",
            Self::InTransit => "in_transit",
            Self::OutForDelivery => "out_for_delivery",
            Self::Delivered => "delivered",
            Self::PartiallyDelivered => "partially_delivered",
            Self::FailedAttempt => "failed_attempt",
            Self::OnHold => "on_hold",
            Self::ReturnInTransit => "return_in_transit",
            Self::Returned => "returned",
            Self::Cancelled => "cancelled",
            Self::Lost => "lost",
            Self::Damaged => "damaged",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl FromStr for CourierStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl From<&str> for CourierStatus {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for CourierStatus {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}

impl fmt::Display for CourierStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for CourierStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CourierStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::parse(&s))
    }
}
