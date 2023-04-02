mod commands;
pub mod formatting;

use async_trait::async_trait;
use serenity::{
    model::{
        application::command::Command,
        gateway::Ready,
        prelude::{interaction::Interaction, *},
    },
    prelude::*,
};
use siege_api::auth::Auth;
use std::{collections::HashMap, sync::Arc};

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use commands::CommandHandler;
use uuid::Uuid;

use crate::commands::{
    id::IdCommand, operator::OperatorCommand, ping::PingCommand, statistics::StatisticsCommand,
    CommandError,
};

struct Handler;

async fn sync_commands(guild_id: GuildId, ctx: &Context) {
    tracing::info!("Syncing commands to {guild_id}");
    // match GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
    //     commands
    //         .create_application_command(|command| PingCommand::register(command))
    //         .create_application_command(|command| IdCommand::register(command))
    //         .create_application_command(|command| StatisticsCommand::register(command))
    //         .create_application_command(|command| OperatorCommand::register(command))
    // })
    // .await
    // {
    //     Ok(commands) => tracing::trace!("Create guild slash commands: {commands:#?}"),
    //     Err(err) => tracing::error!("Failed to create guild commands: {err:#?}"),
    // };

    match Command::set_global_application_commands(&ctx.http, |commands| {
        commands
            .create_application_command(|command| PingCommand::register(command))
            .create_application_command(|command| IdCommand::register(command))
            .create_application_command(|command| StatisticsCommand::register(command))
            .create_application_command(|command| OperatorCommand::register(command))
    })
    .await
    {
        Ok(commands) => tracing::trace!("Created global slash commands: {commands:#?}"),
        Err(err) => tracing::error!("Failed to create commands: {err:#?}"),
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
                    name => tracing::warn!("Autocomplete for {name} not handled"),
                }
            }
            _ => tracing::warn!("Unhanled interation: {interaction:?}"),
        }
    }
}

struct SiegeApi;

impl TypeMapKey for SiegeApi {
    type Value = siege_api::client::Client;
}

struct SiegePlayerLookup;

impl TypeMapKey for SiegePlayerLookup {
    type Value = Arc<RwLock<HashMap<UserId, Uuid>>>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("siege_bot=debug".parse()?))
        .init();

    let token = std::env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();

    {
        let siege_client: siege_api::client::Client =
            Auth::from_environment().connect().await.unwrap().into();
        let mut data = client.data.write().await;
        data.insert::<SiegeApi>(siege_client);
    }
    {
        // TODO: Serialize to disk to be persisted
        let mut lookup = HashMap::new();
        lookup.insert(
            UserId::from(394273324236144641),
            Uuid::parse_str("e7679633-31ff-4f44-8cfd-d0ff81e2c10a").expect("this is a valid guid"),
        );
        let mut data = client.data.write().await;
        data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(lookup)));
    }

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    tracing::info!("Starting client");
    // Finally, start a single shard, and start listening to events.
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
