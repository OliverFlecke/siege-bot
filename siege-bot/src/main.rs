mod commands;

use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::GuildId;
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::Guild};

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _ready: Ready) {
        tracing::info!("Client is ready");
    }

    async fn guild_create(&self, ctx: Context, guild: Guild) {
        tracing::info!("Connecting to guild: {:?}", guild.id);

        let guild_id = guild.id;

        // GuildId::get_application_commands(&guild_id, &ctx.http)
        //     .await
        //     .iter()
        //     .for_each(|id| tracing::debug!("Commands: {id:#?}"));
        // GuildId::delete_application_command(&guild_id, &ctx.http));

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::ping::register(command))
                .create_application_command(|command| commands::id::register(command))
            // .create_application_command(|command| commands::welcome::register(command))
            // .create_application_command(|command| commands::numberinput::register(command))
            // .create_application_command(|command| commands::attachmentinput::register(command))
        })
        .await;

        tracing::trace!("Created guild slash commands: {commands:#?}");

        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::ping::register(command)
        })
        .await;

        tracing::trace!("Created global slash commands: {guild_command:#?}");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            tracing::trace!(
                "Received command interation from {guild_id:?}: {command:#?}",
                guild_id = command.guild_id
            );

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                "id" => commands::id::run(&command.data.options),
                _ => "not implemented".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                tracing::error!("Failed to response to command: {why}");
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(format!("siege_bot=debug").parse()?))
        .init();

    let token = std::env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();

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
