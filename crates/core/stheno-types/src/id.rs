use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use ulid::Ulid;

/// Typed identifier with a prefix matching Medusa v2 conventions (e.g. `prod_01J...`).
pub struct Id<T> {
    inner: Ulid,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Self {
            inner: Ulid::new(),
            _marker: PhantomData,
        }
    }

    pub fn from_ulid(ulid: Ulid) -> Self {
        Self {
            inner: ulid,
            _marker: PhantomData,
        }
    }

    pub fn from_string(s: impl AsRef<str>) -> std::result::Result<Self, crate::Error> {
        let str_ref = s.as_ref();
        // Strip prefix if formatted like prod_01J...
        let clean = if let Some((_, rest)) = str_ref.split_once('_') {
            rest
        } else {
            str_ref
        };
        let ulid = Ulid::from_string(clean).map_err(|e| crate::Error::Invalid {
            code: "invalid_id",
            message: e.to_string(),
            field: Some("id".to_string()),
        })?;
        Ok(Self::from_ulid(ulid))
    }

    pub fn ulid(&self) -> Ulid {
        self.inner
    }
}

impl<T> FromStr for Id<T> {
    type Err = crate::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

impl<T> Default for Id<T> {
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

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.inner)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner.to_string())
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let clean = if let Some((_, rest)) = s.split_once('_') {
            rest
        } else {
            &s
        };
        let ulid = Ulid::from_string(clean).map_err(serde::de::Error::custom)?;
        Ok(Self::from_ulid(ulid))
    }
}
