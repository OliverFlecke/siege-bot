use lazy_static::lazy_static;
use std::fmt::Display;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the different platforms that it is possible to play Siege on.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformType {
    Uplay,
    // These have not been verified
    #[serde(rename = "xbl")]
    Xbox,
    #[serde(rename = "psn")]
    Playstation,
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
            PlatformType::Playstation => &PLAYSTATION_SPACE,
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
            PlatformType::Playstation => &PLAYSTATION,
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

/// The full profile returned from the Ranked V2 API. This, together with its
/// nested fields, contains the high level data for each season.
#[derive(Debug, Deserialize, Getters, Clone, Copy)]
pub struct FullProfile {
    profile: Profile,
    season_statistics: SeasonStatistics,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy)]
pub struct Profile {
    #[serde(rename = "board_id")]
    play_type: PlayType,
    id: Uuid,
    max_rank: i64,
    max_rank_points: i64,
    platform_family: PlatformFamily,
    season_id: u8,
    top_rank_position: i64,
}

#[derive(Debug, Deserialize, Getters, Clone, Copy)]
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

#[derive(Debug, Deserialize, Getters, Clone, Copy)]
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

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PlatformFamily {
    Pc,
    Console,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PlayType {
    Casual,
    Ranked,
    Event,
    Warmup,
}

pub use operator::*;

mod operator {
    use crate::operator::Operator;

    use super::*;

    #[derive(Debug)]
    pub enum SideOrAll {
        All,
        Attacker,
        Defender,
    }

    #[derive(Debug, Deserialize, Getters)]
    #[serde(rename_all = "camelCase")]
    pub struct OperatorStatisticResponse {
        profile_id: Uuid,
        #[serde(deserialize_with = "mappers::int_to_naive_date")]
        start_date: NaiveDate,
        #[serde(deserialize_with = "mappers::int_to_naive_date")]
        end_date: NaiveDate,
        region: String,
        stat_type: String,

        #[getter(skip)]
        platforms: Platforms,
    }

    impl OperatorStatisticResponse {
        pub fn get_operators(&self, role: SideOrAll) -> Option<&Vec<OperatorStatistics>> {
            let roles = match self.platforms.pc.game_modes.all.as_ref() {
                Some(r) => r,
                None => return None,
            };

            match role {
                SideOrAll::All => Some(&roles.team_roles.all),
                SideOrAll::Attacker => Some(&roles.team_roles.attacker),
                SideOrAll::Defender => Some(&roles.team_roles.defenders),
            }
        }

        /// Get an operator with a specific name.
        pub fn get_operator(&self, operator: Operator) -> Option<&OperatorStatistics> {
            self.get_operators(SideOrAll::All)
                .and_then(|x| x.iter().find(|op| op.name == operator))
        }
    }

    #[derive(Debug, Deserialize)]
    struct Platforms {
        #[serde(rename = "PC")]
        pc: OperatorResponsePlatform,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OperatorResponsePlatform {
        game_modes: GameModes,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct GameModes {
        all: Option<Mode>,
        #[allow(dead_code)]
        ranked: Option<Mode>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Mode {
        team_roles: Roles,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Roles {
        all: Vec<OperatorStatistics>,
        #[serde(rename = "Defender")]
        defenders: Vec<OperatorStatistics>,
        #[serde(rename = "Attacker")]
        attacker: Vec<OperatorStatistics>,
    }

    #[derive(Debug, Deserialize, Getters)]
    #[serde(rename_all = "camelCase")]
    pub struct OperatorStatistics {
        // type: String, // Seems to always be `Generalized`
        // stats_type: String, // Seems to always be `operators`
        #[serde(rename = "statsDetail")]
        name: Operator,
        matches_played: u64,
        rounds_played: u64,
        minutes_played: u64,
        matches_won: u64,
        matches_lost: u64,
        rounds_won: u64,
        rounds_lost: u64,
        kills: u64,
        assists: u64,
        #[serde(rename = "death")]
        deaths: u64,
        headshots: u64,
        melee_kills: u64,
        team_kills: u64,
        opening_kills: u64,
        opening_deaths: u64,
        trades: u64,
        opening_kill_trades: u64,
        opening_death_trades: u64,
        revives: u64,
        distance_travelled: u64,
        win_loss_ratio: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        kill_death_ratio: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        headshot_accuracy: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        kills_per_round: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_with_a_kill: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_with_multi_kill: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_with_opening_kill: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_with_opening_death: f64,
        #[serde(
            deserialize_with = "mappers::extract_nested_float_value",
            rename = "roundsWithKOST"
        )]
        rounds_with_kost: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_survived: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_with_an_ace: f64,
        #[serde(deserialize_with = "mappers::extract_nested_float_value")]
        rounds_with_clutch: f64,
        #[serde(deserialize_with = "mappers::float_to_duration")]
        time_alive_per_match: Duration,
        #[serde(deserialize_with = "mappers::float_to_duration")]
        time_dead_per_match: Duration,
        distance_per_round: f64,
    }

    impl OperatorStatistics {
        /// Get a URL for the avatar for this operator.
        pub fn avatar_url(&self) -> String {
            format!(
                "https://r6operators.marcopixel.eu/icons/png/{}.png",
                self.name.to_string().to_lowercase()
            )
        }

        pub fn opening_win_rate(&self) -> f64 {
            self.opening_kills as f64 / (self.opening_kills + self.opening_deaths).max(1) as f64
        }

        pub fn matches_win_rate(&self) -> f64 {
            self.matches_won as f64 / self.matches_played as f64
        }

        pub fn rounds_win_rate(&self) -> f64 {
            self.rounds_won as f64 / self.rounds_played as f64
        }
    }
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
}
