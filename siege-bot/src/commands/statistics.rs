use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    utils::Color,
};

use crate::SiegeApi;

use super::{
    context::DiscordContext, discord_app_command::DiscordAppCmd, AddUserOptionToCommand, CmdResult,
    CommandHandler,
};

pub struct StatisticsCommand;

#[async_trait]
impl CommandHandler for StatisticsCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("statistics")
            .description("Get the statistics for a Siege player")
            .add_user_option()
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let user = command.get_user_from_command_or_default();
        let player_id = ctx.lookup_siege_player(command, &user).await?;

        tracing::info!("Getting statistics for {}", user.name);

        let profiles = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            siege_client.get_full_profiles(player_id).await.unwrap()
        };

        let profile: siege_api::models::FullProfile = profiles[0];

        let season = profile.season_statistics().clone();
        let matches = season.match_outcomes().clone();

        command
            .send_embedded(
                ctx.http().clone(),
                CreateEmbed::default()
                    .title(format!("Statistics for {}", user.name))
                    .thumbnail(user.avatar_url().unwrap())
                    .field(
                        "Kill/death",
                        format!(
                            "K/D: **{kd:.2}** - Kills {kills} / Deaths {deaths}",
                            kd = season.kd(),
                            kills = season.kills(),
                            deaths = season.deaths(),
                        ),
                        false,
                    )
                    .field(
                        "Match",
                        format!(
                            "Matches {total} - **{win_rate:.2} %** - Wins **{wins}** / Losses **{losses}**",
                            total = matches.total_matches(),
                            wins = matches.wins(),
                            losses = matches.losses(),
                            win_rate = matches.win_rate() * 100.0,
                        ),
                        false,
                    )
                    .color(Color::DARK_RED)
                    .clone(),
            )
            .await
    }
}
