use serenity::{builder::CreateEmbed, model::Timestamp};
use siege_api::models::MapStatistics;

use super::FormatEmbedded;

impl FormatEmbedded<'_, Vec<&MapStatistics>> for CreateEmbed {
    fn format(&mut self, maps: &Vec<&MapStatistics>) -> &mut Self {
        self.timestamp(Timestamp::now());

        let names = maps
            .iter()
            .map(|x| format!("`{}`", x.name()))
            .fold(String::new(), |acc, next| acc + &next + "\n");
        let kds = maps
            .iter()
            .map(|x| x.statistics().kill_death_ratio())
            .map(|kd| format!("`{kd:.2}`"))
            .fold(String::new(), |acc, next| acc + &next + "\n");
        let rounds = maps
            .iter()
            .map(|x| {
                format!(
                    "M: `{:.2} %` (`{}`) R: `{:.2} %` (`{}`)",
                    100.0 * x.statistics().matches_win_rate(),
                    x.statistics().matches_played(),
                    100.0 * x.statistics().rounds_win_rate(),
                    x.statistics().rounds_played()
                )
            })
            .fold(String::new(), |acc, next| acc + &next + "\n");

        self.field("Map", names, true);
        self.field("K/D", kds, true);
        self.field("Winrate and total", rounds, true);

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
        let content = std::fs::read_to_string("../samples/maps.json").unwrap();
        let stats: StatisticResponse = serde_json::from_str(content.as_str()).unwrap();
        let maps = stats.get_maps(AllOrRanked::All, SideOrAll::All);

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
