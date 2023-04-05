use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::Context,
    utils::Color,
};

use crate::{commands::CommandError, SiegeApi};

use super::{
    get_user_from_command_or_default, lookup_siege_player, AddUserOptionToCommand, CommandHandler,
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

    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), super::CommandError> {
        let data = ctx.data.read().await;
        let siege_client = data
            .get::<SiegeApi>()
            .expect("Siege client is always registered");

        let user = get_user_from_command_or_default(command);
        let player_id = lookup_siege_player(ctx, command, user).await?;
        let profiles = siege_client.get_full_profiles(player_id).await.unwrap();
        let profile: siege_api::models::FullProfile = profiles[0];

        let season = profile.season_statistics();
        let matches = season.match_outcomes();

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|msg| {
                        msg.embed(|e| {
                            e.title(format!("Statistics for {}", user.name))
                                .thumbnail(user.avatar_url().unwrap())
                                .field(
                                    "Kill/death",
                                    format!(
                                        "K/D: **{kd:.2}** - Kills {kills} / Deaths {deaths}",
                                        kd = season.kd( ),
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
                        })
                    })
            })
            .await
            .map_err(CommandError::SerenityError)?;

        Ok(())
    }
}
