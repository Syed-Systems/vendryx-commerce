use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use ulid::Ulid;

/// The Medusa identifier prefix for an entity type (ADR-010).
///
/// The prefix is part of the wire contract, not decoration: Medusa serialises
/// `prod_01J...`, and a client that round-trips an id back to us sends the same
/// string. Making it a compile-time constant keeps `Id<T>` a bare ULID in memory
/// while still rendering the full form on every boundary.
pub trait Prefixed {
    const PREFIX: &'static str;
}

/// Declares `Prefixed` impls as a table.
///
/// Prefixes are wire contract; a wrong entry is a compatibility bug. Keep this a
/// flat list so it can be read against Medusa's model definitions in one pass.
#[macro_export]
macro_rules! prefixed_id {
    ($($ty:ty => $prefix:literal),+ $(,)?) => {
        $(impl $crate::id::Prefixed for $ty {
            const PREFIX: &'static str = $prefix;
        })+
    };
}

/// Typed identifier rendering as `{prefix}_{ulid}` (e.g. `prod_01J...`).
pub struct Id<T> {
    inner: Ulid,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn from_ulid(ulid: Ulid) -> Self {
        Self {
            inner: ulid,
            _marker: PhantomData,
        }
    }

    pub fn ulid(&self) -> Ulid {
        self.inner
    }
}

impl<T: Prefixed> Id<T> {
    pub fn new() -> Self {
        Self::from_ulid(Ulid::new())
    }

    /// The prefix this id renders with.
    pub fn prefix() -> &'static str {
        T::PREFIX
    }

    /// Parses `{prefix}_{ulid}`.
    ///
    /// A mismatched prefix is rejected rather than ignored: ids arrive on HTTP
    /// paths, and `GET /admin/products/cart_01J...` is a client error, not a
    /// lookup that happens to miss (ADR-010).
    pub fn from_string(s: impl AsRef<str>) -> std::result::Result<Self, crate::Error> {
        let raw = s.as_ref();
        let ulid_part = raw
            .strip_prefix(T::PREFIX)
            .and_then(|rest| rest.strip_prefix('_'))
            .ok_or_else(|| crate::Error::Invalid {
                code: "invalid_id",
                message: format!(
                    "expected an id prefixed with '{}_', got '{}'",
                    T::PREFIX,
                    raw
                ),
                field: Some("id".to_string()),
            })?;

        let ulid = Ulid::from_string(ulid_part).map_err(|e| crate::Error::Invalid {
            code: "invalid_id",
            message: e.to_string(),
            field: Some("id".to_string()),
        })?;
        Ok(Self::from_ulid(ulid))
    }
}

impl<T: Prefixed> FromStr for Id<T> {
    type Err = crate::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

impl<T: Prefixed> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: Prefixed> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({}_{})", T::PREFIX, self.inner)
    }
}

impl<T: Prefixed> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", T::PREFIX, self.inner)
    }
}

impl<T: Prefixed> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de, T: Prefixed> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_string(&s).map_err(serde::de::Error::custom)
    }
}
