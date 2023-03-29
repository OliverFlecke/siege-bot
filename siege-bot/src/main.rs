mod commands;

use async_trait::async_trait;
use serenity::{
    model::{
        application::command::Command,
        gateway::Ready,
        prelude::{interaction::Interaction, *},
    },
    prelude::*,
};

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use commands::CommandHandler;

use crate::commands::{
    id::IdCommand, ping::PingCommand, statistics::StatisticsCommand, CommandError,
};

struct Handler;

async fn sync_commands(guild_id: GuildId, ctx: &Context) {
    let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
        commands
            .create_application_command(|command| PingCommand::register(command))
            .create_application_command(|command| IdCommand::register(command))
            .create_application_command(|command| StatisticsCommand::register(command))
    })
    .await;

    tracing::trace!("Created guild slash commands: {commands:#?}");

    let guild_command = Command::create_global_application_command(&ctx.http, |command| {
        PingCommand::register(command);
        IdCommand::register(command);
        StatisticsCommand::register(command);

        command
    })
    .await;

    tracing::trace!("Created global slash commands: {guild_command:#?}");
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
        if let Interaction::ApplicationCommand(command) = interaction {
            tracing::trace!(
                "Received command interation from {guild_id:?}: {command:#?}",
                guild_id = command.guild_id
            );

            let result = match command.data.name.as_str() {
                "ping" => PingCommand::run(&ctx, &command).await,
                "id" => IdCommand::run(&ctx, &command).await,
                "statistics" => StatisticsCommand::run(&ctx, &command).await,
                _ => Err(CommandError::CommandNotFound),
            };

            if let Err(why) = result {
                tracing::error!("Failed to response to command: {why}");
            }
        }
    }
}

struct SiegeApi;

impl TypeMapKey for SiegeApi {
    type Value = siege_api::client::Client;
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
        let mut data = client.data.write().await;
        let client: siege_api::client::Client = siege_api::auth::Auth::from_environment()
            .connect()
            .await
            .unwrap()
            .into();
        data.insert::<SiegeApi>(client);
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
