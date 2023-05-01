use serenity::{builder::CreateEmbed, model::Timestamp};
use siege_api::models::OperatorStatistics;

use super::FormatEmbedded;

impl FormatEmbedded<'_, Vec<&OperatorStatistics>> for CreateEmbed {
    fn format(&mut self, operators: &Vec<&OperatorStatistics>) -> &mut Self {
        self.timestamp(Timestamp::now());

        let names = operators
            .iter()
            .map(|op| format!("`{}`", op.name()))
            .fold(String::new(), |acc, next| acc + &next + "\n");
        let kds = operators
            .iter()
            .map(|op| op.statistics().kill_death_ratio())
            .map(|kd| format!("`{kd:.2}`"))
            .fold(String::new(), |acc, next| acc + &next + "\n");
        let rounds = operators
            .iter()
            .map(|op| {
                format!(
                    "`{: >6.2} %` (of `{: >3}`)",
                    100.0 * op.statistics().rounds_win_rate(),
                    op.statistics().rounds_played()
                )
            })
            .fold(String::new(), |acc, next| acc + &next + "\n");

        self.field("Operator", names, true);
        self.field("K/D", kds, true);
        self.field("Rounds", rounds, true);

        self
    }
}

#[cfg(test)]
mod test {
    use std::ops::Sub;

    use chrono::{DateTime, Utc};
    use siege_api::models::{AllOrRanked, SideOrAll, StatisticResponse};

    use super::*;

    #[test]
    fn format_validate() {
        let mut embed = CreateEmbed::default();
        let content = std::fs::read_to_string("../samples/operators.json").unwrap();
        let stats: StatisticResponse = serde_json::from_str(content.as_str()).unwrap();
        let maps = stats.get_operators(AllOrRanked::All, SideOrAll::All);

        embed.format(&maps);

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
