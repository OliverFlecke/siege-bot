use serenity::{builder::CreateEmbed, model::Timestamp};
use siege_api::models::Statistics;

use super::FormatEmbedded;

/// Create an embedded Discord message with statistics information.
impl FormatEmbedded<'_, Statistics> for CreateEmbed {
    fn format(&mut self, statistics: &Statistics) -> &mut Self {
        let values = vec![
            ("Minutes played", statistics.minutes_played().to_string()),
            (
                "Rounds with a kill",
                format!("{:.2} %", 100f64 * statistics.rounds_with_a_kill()),
            ),
            (
                "Rounds with KOST",
                format!("{:.2} %", 100f64 * statistics.rounds_with_kost()),
            ),
            (
                "Rounds with a multi kills",
                format!("{:.2} %", 100f64 * statistics.rounds_with_multi_kill()),
            ),
            (
                "Rounds with opening kill",
                format!(
                    "{:.2} % ({})",
                    100f64 * statistics.rounds_with_opening_kill(),
                    statistics.opening_kills(),
                ),
            ),
            (
                "Rounds with opening death",
                format!(
                    "{:.2} % ({})",
                    100f64 * statistics.rounds_with_opening_death(),
                    statistics.opening_deaths(),
                ),
            ),
            (
                "Opening winrate",
                format!("{:.2} %", 100f64 * statistics.opening_win_rate()),
            ),
            (
                "Rounds survived",
                format!("{:.2} %", 100f64 * statistics.rounds_survived()),
            ),
            (
                "Rounds with an ace",
                format!("{:.2} %", 100f64 * statistics.rounds_with_an_ace()),
            ),
            (
                "Rounds with a clutch",
                format!("{:.2} %", 100f64 * statistics.rounds_with_clutch()),
            ),
            (
                "Kills per round",
                format!("{:.2} %", 100f64 * statistics.kills_per_round()),
            ),
            (
                "Headshots",
                format!(
                    "{:.2} % ({})",
                    100f64 * statistics.headshot_accuracy(),
                    statistics.headshots(),
                ),
            ),
            ("Melee kills", statistics.melee_kills().to_string()),
            ("Team kills", statistics.team_kills().to_string()),
            ("Trades", statistics.trades().to_string()),
            ("Revives", statistics.revives().to_string()),
            (
                "Time alive per match",
                statistics.time_alive_per_match().to_string(),
            ),
            (
                "Time dead per match",
                statistics.time_dead_per_match().to_string(),
            ),
        ];

        let names = values.iter().map(|x| x.0).collect::<Vec<_>>().join("\n");
        let values = values
            .iter()
            .map(|x| x.1.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        self
            .timestamp(Timestamp::now())
            .field(
                "Kills",
                format!(
                    "K/D: **{kd:.2}** - KDA: **{kills}** / **{deaths}** / **{assists}**",
                    kd = statistics.kill_death_ratio(),
                    kills = statistics.kills(),
                    deaths = statistics.deaths(),
                    assists = statistics.assists(),
                 ),
                false,
            )
            .field(
                "Matches",
                format!(
                    "Win rate **{win_rate:.2} %** Played/Win/Lost: **{total}** / **{wins}** / **{lost}**",
                    win_rate = 100f64 * statistics.matches_win_rate(),
                    total = statistics.matches_played(),
                    wins = statistics.matches_won(),
                    lost = statistics.matches_lost(),
                ),
                false,
            )
            .field(
                "Rounds",
                format!(
                    "Win rate **{win_rate:.2} %** Played/Win/Lost: **{total}** / **{wins}** / **{lost}**",
                    win_rate = 100f64 * statistics.rounds_win_rate(),
                    total = statistics.rounds_played(),
                    wins = statistics.rounds_won(),
                    lost = statistics.rounds_lost(),
                ),
                false,
            )
            .field("Statistic", names, true)
            .field("Value", values, true)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Sub;

    use chrono::{DateTime, Utc};
    use siege_api::{
        models::{AllOrRanked, StatisticResponse},
        operator,
    };

    use super::*;

    #[test]
    fn format_validate() {
        let mut embed = CreateEmbed::default();
        let content = std::fs::read_to_string("../samples/operators.json").unwrap();
        let stats: StatisticResponse = serde_json::from_str(content.as_str()).unwrap();
        let operator = stats
            .get_operator(operator::Operator::Hibana, AllOrRanked::All)
            .unwrap();

        embed.format(operator.statistics());

        assert!(
            embed
                .0
                .get("timestamp")
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<DateTime<Utc>>()
                .unwrap()
                .sub(Utc::now())
                .num_seconds()
                < 1
        );

        println!("{embed:?}");
    }
}
