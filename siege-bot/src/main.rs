mod commands;
mod constants;
pub mod formatting;
pub mod handler;
pub mod siege_player_lookup;

use crate::{handler::Handler, siege_player_lookup::PlayerLookupImpl};
use serenity::{
    model::prelude::*,
    prelude::{RwLock, TypeMapKey},
    Client,
};
use siege_api::auth::Auth;
use siege_player_lookup::SiegePlayerLookup;
use std::{env::var, error::Error, sync::Arc};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

struct SiegeApi;
impl TypeMapKey for SiegeApi {
    type Value = Arc<dyn siege_api::client::SiegeClient>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup tracing
    let file_appender = tracing_appender::rolling::never(
        var("LOGS_DIR").unwrap_or_else(|_| "./logs/".to_string()),
        "siege-bot.log",
    );
    // `_guard` is needed to ensure logs are flush when dropped.
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer())
        .with(
            EnvFilter::from_default_env()
                .add_directive("siege_bot=debug".parse()?)
                .add_directive("siege_api=info".parse()?),
        )
        .init();

    let token = var("DISCORD_TOKEN").expect("environment variable `DISCORD_TOKEN` should be set");
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    setup_type_map(&mut client).await?;

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.shutdown_all().await;
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

async fn setup_type_map(client: &mut Client) -> Result<(), Box<dyn Error>> {
    // These extra scopes are added to drop the `GuardLock` on `data` as soon
    // as possible. This not strictly necessary here, but in general best
    // pratice to hold the locks as shortly as possible.
    {
        let siege_client: siege_api::client::Client =
            Auth::from_environment().connect().await.unwrap().into();
        let mut data = client.data.write().await;
        data.insert::<SiegeApi>(Arc::new(siege_client));
    }
    {
        let lookup = PlayerLookupImpl::load(".players.json")?;
        let mut data = client.data.write().await;
        data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(lookup)));
    }

    Ok(())
}
