use lazy_static::lazy_static;
use std::fmt::Display;
use strum::{Display, EnumIter, EnumString};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use playerstats::*;

pub mod meta;
/// This section contains all models related to the `playerstats` endpoint
mod playerstats;

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    profile_id: Uuid,
    user_id: Uuid,
    platform_type: PlatformType,
    id_on_platform: Uuid,
    name_on_platform: String,
}

/// Represents the different platforms that it is possible to play Siege on.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum PlatformType {
    Uplay,
    // These have not been verified
    #[serde(rename = "xbl")]
    Xbox,
    #[serde(rename = "psn")]
    PlayStation,
}

impl Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PlatformType {
    /// Get the space this platform is using when quering Ubisoft's API.
    pub fn get_space(&self) -> &Uuid {
        lazy_static! {
            static ref UPLAY_SPACE: Uuid =
                Uuid::parse_str("5172a557-50b5-4665-b7db-e3f2e8c5041d").expect("is valid uuid");
            static ref PLAYSTATION_SPACE: Uuid =
                Uuid::parse_str("05bfb3f7-6c21-4c42-be1f-97a33fb5cf66").expect("is valid uuid");
            static ref XBOX_SPACE: Uuid =
                Uuid::parse_str("98a601e5-ca91-4440-b1c5-753f601a2c90").expect("is valid uuid");
        }

        match self {
            PlatformType::Uplay => &UPLAY_SPACE,
            PlatformType::Xbox => &XBOX_SPACE,
            PlatformType::PlayStation => &PLAYSTATION_SPACE,
        }
    }

    /// Get the sandbox associated with the given platform.
    pub fn get_sandbox(&self) -> &str {
        lazy_static! {
            static ref UPLAY: &'static str = "OSBOR_PC_LNCH_A";
            static ref PLAYSTATION: &'static str = "OSBOR_PS4_LNCH_A";
            static ref XBOX: &'static str = "OSBOR_XBOXONE_LNCH_A";
        }

        match self {
            PlatformType::Uplay => &UPLAY,
            PlatformType::Xbox => &XBOX,
            PlatformType::PlayStation => &PLAYSTATION,
        }
    }
}

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

// Helper structs to extract the unnecessary nesting from the API.
#[derive(Debug, Deserialize)]
pub struct RankedV2Response {
    platform_families_full_profiles: Vec<PlatformFamiliesFullProfile>,
}

impl RankedV2Response {
    /// Retreive statistics by platform family.
    pub fn get_for_platform(
        &self,
        platform: PlatformFamily,
    ) -> Option<&PlatformFamiliesFullProfile> {
        self.platform_families_full_profiles
            .iter()
            .find(|x| x.platform_family == platform)
    }

    /// Get the statistics board for a given platform family and play type.
    pub fn get_board(&self, platform: PlatformFamily, play_type: GameMode) -> Option<&FullProfile> {
        self.get_for_platform(platform)
            .and_then(|x| x.get_by_playtype(play_type))
            .and_then(|x| x.full_profiles.get(0))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PlatformFamiliesFullProfile {
    platform_family: PlatformFamily,
    board_ids_full_profiles: Vec<Board>,
}

impl PlatformFamiliesFullProfile {
    pub fn get_by_playtype(&self, play_type: GameMode) -> Option<&Board> {
        self.board_ids_full_profiles
            .iter()
            .find(|x| x.game_mode == play_type)
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Board {
    #[serde(rename = "board_id")]
    game_mode: GameMode,
    full_profiles: Vec<FullProfile>,
}

/// The full profile returned from the Ranked V2 API. This, together with its
/// nested fields, contains the high level data for each season.
#[derive(Debug, Deserialize, Getters, Clone, Copy, PartialEq, Eq)]
pub struct FullProfile {
    profile: Profile,
    season_statistics: SeasonStatistics,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    #[serde(rename = "board_id")]
    game_mode: GameMode,
    id: Uuid,
    max_rank: i64,
    max_rank_points: i64,
    platform_family: PlatformFamily,
    season_id: u8,
    top_rank_position: i64,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy, PartialEq, Eq)]
pub struct SeasonStatistics {
    deaths: u64,
    kills: u64,
    match_outcomes: MatchOutcomes,
}

impl SeasonStatistics {
    pub fn kd(&self) -> f64 {
        self.kills as f64 / self.deaths as f64
    }
}

#[derive(Debug, Default, Deserialize, Getters, Clone, Copy, PartialEq, Eq)]
pub struct MatchOutcomes {
    abandons: u64,
    losses: u64,
    wins: u64,
}

impl MatchOutcomes {
    /// Get the total number of matches played.
    /// Note that this does **not** include matches that have been abandoned.
    pub fn total_matches(&self) -> u64 {
        self.wins + self.losses
    }

    /// Get the total number of matches played, including abandons.
    pub fn total_matches_with_abandons(&self) -> u64 {
        self.wins + self.losses + self.abandons
    }

    /// Calculate the win rate for the current match outcomes.
    /// Note that this calculation does **not** take abandoned matches into account.
    /// For that see `win_rate_with_abandons`.
    pub fn win_rate(&self) -> f64 {
        self.wins as f64 / self.total_matches() as f64
    }

    pub fn win_rate_with_abandons(&self) -> f64 {
        self.wins as f64 / self.total_matches_with_abandons() as f64
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, EnumString, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum PlatformFamily {
    Pc,
    Console,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, EnumString, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    Casual,
    Ranked,
    Event,
    Warmup,
}

mod mappers {
    use serde::Deserializer;

    use super::*;

    pub fn int_string_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let n: i64 = s.parse().map_err(|_| {
            serde::de::Error::custom(format!(
                "cannot convert string value to an unsinged integer: {s}"
            ))
        })?;

        Ok(Duration::seconds(n))
    }

    pub fn int_to_naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: usize = Deserialize::deserialize(deserializer)?;

        Ok(NaiveDate::parse_from_str(s.to_string().as_str(), "%Y%m%d").unwrap())
    }

    pub fn extract_nested_float_value<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let item: PercentValue = Deserialize::deserialize(deserializer)?;

        Ok(item.value)
    }

    #[derive(Debug, Deserialize)]
    struct PercentValue {
        value: f64,
    }

    pub fn float_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let item: f64 = Deserialize::deserialize(deserializer)?;
        let duration = Duration::milliseconds((item * 1000f64).round() as i64);

        Ok(duration)
    }

    pub fn string_to_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: &str = Deserialize::deserialize(deserializer)?;
        if value.is_empty() {
            Ok(None)
        } else {
            Uuid::parse_str(value).map(Some).map_err(|err| {
                serde::de::Error::custom(format!(
                    "cannot convert string value '{value}' to an uuid. Err: {err}"
                ))
            })
        }
    }

    #[cfg(test)]
    mod test {
        use serde::de::{
            value::{Error, StrDeserializer, StringDeserializer},
            IntoDeserializer,
        };

        use super::*;

        #[test]
        fn valid_string_to_duration() {
            let value = "1234".to_string();
            let des: StringDeserializer<Error> = value.into_deserializer();

            assert_eq!(int_string_to_duration(des), Ok(Duration::seconds(1234)));
        }

        #[test]
        fn invalid_string_to_duration() {
            let des: StrDeserializer<Error> = "not a number".into_deserializer();
            assert!(int_string_to_duration(des).is_err());
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn ranked_v2_get() {
        let content = read_to_string("../samples/full_profile.json").unwrap();
        let response: RankedV2Response = serde_json::from_str(content.as_str()).unwrap();

        assert_eq!(
            response.get_for_platform(PlatformFamily::Pc),
            response.platform_families_full_profiles.get(0),
        );

        // No console data is included in the sample, so `None` is expected.
        assert_eq!(response.get_for_platform(PlatformFamily::Console), None);
    }

    #[test]
    fn ranked_v2_get_board_by_play_type() {
        let content = read_to_string("../samples/full_profile.json").unwrap();
        let response: RankedV2Response = serde_json::from_str(content.as_str()).unwrap();

        let platforms = response.platform_families_full_profiles.get(0).unwrap();

        assert_eq!(
            platforms.board_ids_full_profiles.get(0),
            platforms.get_by_playtype(GameMode::Casual)
        );
        assert_eq!(
            platforms.board_ids_full_profiles.get(1),
            platforms.get_by_playtype(GameMode::Event)
        );
        assert_eq!(
            platforms.board_ids_full_profiles.get(2),
            platforms.get_by_playtype(GameMode::Warmup)
        );
        assert_eq!(
            platforms.board_ids_full_profiles.get(3),
            platforms.get_by_playtype(GameMode::Ranked)
        );
    }

    #[test]
    fn ranked_v2_get_board_by_platform_and_play_type() {
        let content = read_to_string("../samples/full_profile.json").unwrap();
        let response: RankedV2Response = serde_json::from_str(content.as_str()).unwrap();

        let expected = response
            .platform_families_full_profiles
            .get(0)
            .and_then(|x| x.board_ids_full_profiles.get(3))
            .and_then(|x| x.full_profiles.get(0));

        assert_eq!(
            response.get_board(PlatformFamily::Pc, GameMode::Ranked),
            expected
        );
    }

    #[test]
    fn get_spaces() {
        PlatformType::iter().for_each(|x| {
            x.get_space();
        });
    }

    #[test]
    fn get_sandboxes() {
        PlatformType::iter().for_each(|x| {
            x.get_sandbox();
        });
    }

    // SeasonStatistics
    #[test]
    fn kd_on_season_statistics() {
        let stats = SeasonStatistics {
            kills: 124,
            deaths: 100,
            match_outcomes: MatchOutcomes::default(),
        };

        assert_eq!(stats.kd(), 1.24);
    }

    #[test]
    fn win_rates_on_match_outcomes() {
        let outcomes = MatchOutcomes {
            abandons: 1,
            losses: 10,
            wins: 12,
        };

        assert_eq!(outcomes.total_matches(), 22);
        assert_eq!(outcomes.total_matches_with_abandons(), 23);
        assert_eq!(outcomes.win_rate(), 0.5454545454545454);
        assert_eq!(outcomes.win_rate_with_abandons(), 0.5217391304347826);
    }
}
