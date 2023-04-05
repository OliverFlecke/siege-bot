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
                    "`{:.2} %` (of `{}`)",
                    op.statistics().rounds_win_rate(),
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
