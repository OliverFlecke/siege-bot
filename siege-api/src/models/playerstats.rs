use std::str::FromStr;

use crate::{game_models::Season, operator::Operator};

use super::*;

#[derive(Debug)]
pub enum SideOrAll {
    All,
    Attacker,
    Defender,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct StatisticResponse {
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

impl StatisticResponse {
    pub fn get_operators(&self, role: SideOrAll) -> Option<&Vec<GeneralStatistics>> {
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
        self.get_operators(SideOrAll::All).and_then(|x| {
            x.iter()
                .filter_map(|x| match x {
                    GeneralStatistics::Operator(op) => Some(op),
                    _ => None,
                })
                .find(|op| op.name == operator)
        })
    }

    /// Get statistics for a given map.
    pub fn get_map(&self, map_name: &str) -> Option<&MapStatistics> {
        self.get_operators(SideOrAll::All).and_then(|stats| {
            stats
                .iter()
                .filter_map(|x| match x {
                    GeneralStatistics::Maps(map) => Some(map),
                    _ => None,
                })
                .find(|map| map.name().as_str() == map_name.to_uppercase())
        })
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
    all: Vec<GeneralStatistics>,
    #[serde(rename = "Defender")]
    defenders: Vec<GeneralStatistics>,
    #[serde(rename = "Attacker")]
    attacker: Vec<GeneralStatistics>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "statsType")]
pub enum GeneralStatistics {
    #[serde(rename = "operators")]
    Operator(OperatorStatistics),
    #[serde(rename = "summary")]
    Summary(SeasonalStatistics),
    #[serde(rename = "maps")]
    Maps(MapStatistics),
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct OperatorStatistics {
    #[serde(rename = "statsDetail")]
    name: Operator,

    #[serde(flatten)]
    statistics: Statistics,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SeasonalStatistics {
    #[getter(skip)]
    season_year: String,
    #[getter(skip)]
    season_number: String,

    #[serde(flatten)]
    statistics: Statistics,
}

impl SeasonalStatistics {
    pub fn get_season(&self) -> Season {
        Season::from_str(&format!("{}{}", self.season_year, self.season_number))
            .expect("should always be valid")
    }
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct MapStatistics {
    #[serde(rename = "statsDetail")]
    name: String, // TODO: Could be converted to an enum

    #[serde(flatten)]
    statistics: Statistics,
}

/// General statistics that are provided from the `playerstats` endpoint.
#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    #[serde(rename = "type")]
    statistic_type: String, // Seems be `Generalized` for operators and `Seasonal` for seasonal stats
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

impl Statistics {
    pub fn opening_win_rate(&self) -> f64 {
        self.opening_kills as f64 / (self.opening_kills + self.opening_deaths).max(1) as f64
    }

    /// Calculate the win rate of matches. This will always return a number between 0 and 1.
    pub fn matches_win_rate(&self) -> f64 {
        self.matches_won as f64 / self.matches_played as f64
    }

    /// Calculate the win rate of rounds. This will always return a number between 0 and 1.
    pub fn rounds_win_rate(&self) -> f64 {
        self.rounds_won as f64 / self.rounds_played as f64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_general_statistics() {
        let sample = r#"{
            "type": "Seasonal",
            "statsType": "summary",
            "statsDetail": "summary",
            "seasonYear": "Y7",
            "seasonNumber": "S3",
            "matchesPlayed": 133,
            "roundsPlayed": 531,
            "minutesPlayed": 1932,
            "matchesWon": 69,
            "matchesLost": 64,
            "roundsWon": 271,
            "roundsLost": 260,
            "kills": 465,
            "assists": 122,
            "death": 334,
            "headshots": 148,
            "meleeKills": 0,
            "teamKills": 1,
            "openingKills": 19,
            "openingDeaths": 29,
            "trades": 24,
            "openingKillTrades": 0,
            "openingDeathTrades": 0,
            "revives": 13,
            "distanceTravelled": 91083,
            "winLossRatio": 1.0781,
            "killDeathRatio": { "value": 1.3922, "p": 0.0 },
            "headshotAccuracy": { "value": 0.3183, "p": 0.0 },
            "killsPerRound": { "value": 0.8757, "p": 0.0 },
            "roundsWithAKill": { "value": 0.5386, "p": 0.0 },
            "roundsWithMultiKill": { "value": 0.2392, "p": 0.0 },
            "roundsWithOpeningKill": { "value": 0.0358, "p": 0.0 },
            "roundsWithOpeningDeath": { "value": 0.0546, "p": 0.0 },
            "roundsWithKOST": { "value": 0.6441, "p": 0.0 },
            "roundsSurvived": { "value": 0.371, "p": 0.0 },
            "roundsWithAnAce": { "value": 0.0019, "p": 0.0 },
            "roundsWithClutch": { "value": 0.0188, "p": 0.0 },
            "timeAlivePerMatch": 444.3609,
            "timeDeadPerMatch": 87.9699,
            "distancePerRound": 171.5311
        }"#;

        let statistic: GeneralStatistics = serde_json::from_str(sample).unwrap();

        match statistic {
            GeneralStatistics::Summary(_) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn deserialize_operator_statistics() {
        let sample = r#"{
            "type": "Generalized",
            "statsType": "operators",
            "statsDetail": "Hibana",
            "matchesPlayed": 3,
            "roundsPlayed": 3,
            "minutesPlayed": 10,
            "matchesWon": 1,
            "matchesLost": 2,
            "roundsWon": 2,
            "roundsLost": 1,
            "kills": 2,
            "assists": 0,
            "death": 3,
            "headshots": 0,
            "meleeKills": 0,
            "teamKills": 0,
            "openingKills": 0,
            "openingDeaths": 0,
            "trades": 0,
            "openingKillTrades": 0,
            "openingDeathTrades": 0,
            "revives": 0,
            "distanceTravelled": 648,
            "winLossRatio": 0.5,
            "killDeathRatio": { "value": 0.6667, "p": 0.0 },
            "headshotAccuracy": { "value": 0.0, "p": 0.0 },
            "killsPerRound": { "value": 0.6667, "p": 0.0 },
            "roundsWithAKill": { "value": 0.6667, "p": 0.0 },
            "roundsWithMultiKill": { "value": 0.0, "p": 0.0 },
            "roundsWithOpeningKill": { "value": 0.0, "p": 0.0 },
            "roundsWithOpeningDeath": { "value": 0.0, "p": 0.0 },
            "roundsWithKOST": { "value": 0.6667, "p": 0.0 },
            "roundsSurvived": { "value": 0.0, "p": 0.0 },
            "roundsWithAnAce": { "value": 0.0, "p": 0.0 },
            "roundsWithClutch": { "value": 0.0, "p": 0.0 },
            "timeAlivePerMatch": 105.3333,
            "timeDeadPerMatch": 28.0,
            "distancePerRound": 216.0
        }"#;

        let statistic: GeneralStatistics = serde_json::from_str(sample).unwrap();

        match statistic {
            GeneralStatistics::Operator(_) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn deserialize_maps_statistics() {
        let sample = r#"{
            "type": "Generalized",
            "statsType": "maps",
            "statsDetail": "YACHT",
            "matchesPlayed": 20,
            "roundsPlayed": 86,
            "minutesPlayed": 294,
            "matchesWon": 8,
            "matchesLost": 12,
            "roundsWon": 40,
            "roundsLost": 46,
            "kills": 75,
            "assists": 16,
            "death": 51,
            "headshots": 13,
            "meleeKills": 0,
            "teamKills": 0,
            "openingKills": 6,
            "openingDeaths": 3,
            "trades": 4,
            "openingKillTrades": 1,
            "openingDeathTrades": 0,
            "revives": 4,
            "distanceTravelled": 14335,
            "winLossRatio": 0.6667,
            "killDeathRatio": { "value": 1.4706, "p": 0.0 },
            "headshotAccuracy": { "value": 0.1733, "p": 0.0 },
            "killsPerRound": { "value": 0.8721, "p": 0.0 },
            "roundsWithAKill": { "value": 0.593, "p": 0.0 },
            "roundsWithMultiKill": { "value": 0.1977, "p": 0.0 },
            "roundsWithOpeningKill": { "value": 0.0698, "p": 0.0 },
            "roundsWithOpeningDeath": { "value": 0.0349, "p": 0.0 },
            "roundsWithKOST": { "value": 0.7093, "p": 0.0 },
            "roundsSurvived": { "value": 0.407, "p": 0.0 },
            "roundsWithAnAce": { "value": 0.0, "p": 0.0 },
            "roundsWithClutch": { "value": 0.0116, "p": 0.0 },
            "timeAlivePerMatch": 432.7,
            "timeDeadPerMatch": 77.5,
            "distancePerRound": 166.686
        }"#;

        let statistic: GeneralStatistics = serde_json::from_str(sample).unwrap();

        match statistic {
            GeneralStatistics::Maps(_) => {}
            _ => assert!(false),
        }
    }
}
