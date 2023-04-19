use async_trait::async_trait;
use serenity::{
    model::prelude::{interaction::Interaction, Guild, GuildId, Ready},
    prelude::*,
};

use crate::commands::{
    add_player::AddPlayerCommand, all_maps::AllMapsCommand, all_operators::AllOperatorCommand,
    game_status::GameStatusCommand, id::IdCommand, map::MapCommand, operator::OperatorCommand,
    ping::PingCommand, statistics::StatisticsCommand, AutocompleteHandler, CommandError,
    CommandHandler,
};

#[derive(Default)]
pub(crate) struct Handler;

async fn sync_commands(guild_id: GuildId, ctx: &Context) {
    tracing::info!("Syncing commands to {guild_id}");
    match GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
        commands
            .create_application_command(PingCommand::register)
            .create_application_command(IdCommand::register)
            .create_application_command(StatisticsCommand::register)
            .create_application_command(MapCommand::register)
            .create_application_command(OperatorCommand::register)
            .create_application_command(AddPlayerCommand::register)
            .create_application_command(AllOperatorCommand::register)
            .create_application_command(AllMapsCommand::register)
            .create_application_command(GameStatusCommand::register)
    })
    .await
    {
        Ok(commands) => tracing::trace!("Create guild slash commands: {commands:#?}"),
        Err(err) => tracing::error!("Failed to create guild commands: {err:#?}"),
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        tracing::info!("Client is ready");
        tracing::trace!("{ready:?}");

        for guild in ready.guilds {
            sync_commands(guild.id, &_ctx).await;
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild) {
        tracing::info!("Connecting to guild: {:?}", guild.id);

        let guild_id = guild.id;

        GuildId::get_application_commands(&guild_id, &ctx.http)
            .await
            .iter()
            .for_each(|id| tracing::debug!("Commands: {id:#?}"));
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                tracing::trace!(
                    "Received command interation from {guild_id:?}: {command:#?}",
                    guild_id = command.guild_id
                );

                let result = match command.data.name.as_str() {
                    "ping" => PingCommand::run(&ctx, &command).await,
                    "id" => IdCommand::run(&ctx, &command).await,
                    "statistics" => StatisticsCommand::run(&ctx, &command).await,
                    "operator" => OperatorCommand::run(&ctx, &command).await,
                    "map" => MapCommand::run(&ctx, &command).await,
                    "add" => AddPlayerCommand::run(&ctx, &command).await,
                    "all_operators" => AllOperatorCommand::run(&ctx, &command).await,
                    "all_maps" => AllMapsCommand::run(&ctx, &command).await,
                    "status" => GameStatusCommand::run(&ctx, &command).await,
                    _ => Err(CommandError::CommandNotFound),
                };

                if let Err(why) = result {
                    tracing::error!("Failed to response to command: {why}");
                }
            }
            Interaction::Autocomplete(autocomplete) => {
                tracing::trace!("Autocomplete request: {autocomplete:#?}");

                match autocomplete.data.name.as_str() {
                    "operator" => {
                        OperatorCommand::handle_autocomplete(&ctx, &autocomplete)
                            .await
                            .unwrap();
                    }
                    "map" => {
                        MapCommand::handle_autocomplete(&ctx, &autocomplete)
                            .await
                            .unwrap();
                    }
                    name => tracing::warn!("Autocomplete for {name} not handled"),
                }
            }
            _ => tracing::warn!("Unhandled interation: {interaction:?}"),
        }
    }
}
