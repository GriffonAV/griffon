use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

pub struct NoParams;

impl<'de> Deserialize<'de> for NoParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Accept any valid JSON payload (null, {}, [], etc.) and discard it
        let _ = IgnoredAny::deserialize(deserializer)?;
        Ok(NoParams)
    }
}
