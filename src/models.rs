use chrono::{DateTime, Duration, Utc};
use derive_getters::Getters;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, Getters)]
pub struct PlaytimeResponse {
    profiles: Vec<PlaytimeProfile>,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeProfile {
    profile_id: Uuid,
    #[serde(rename = "stats")]
    statistics: PlaytimeStatistics,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy)]
pub struct PlaytimeStatistics {
    #[serde(rename = "PPvPTimePlayed")]
    pvp_time_played: Playtime,
    #[serde(rename = "PClearanceLevel")]
    clearance_level: Playtime,
    #[serde(rename = "PPvETimePlayed")]
    pve_time_played: Playtime,
    #[serde(rename = "PTotalTimePlayed")]
    total_time_played: Playtime,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Playtime {
    #[serde(rename = "value", deserialize_with = "mappers::int_string_to_duration")]
    duration: Duration,
    start_date: DateTime<Utc>,
    /// Last time modified - Can also be considered as the last time played.
    last_modified: DateTime<Utc>,
}

mod mappers {
    use serde::Deserializer;

    use super::*;

    pub fn int_string_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let n: i64 = s.parse().map_err(|_| {
            serde::de::Error::custom(format!(
                "cannot convert string value to an unsinged integer: {s}"
            ))
        })?;

        Ok(Duration::seconds(n))
    }
}
