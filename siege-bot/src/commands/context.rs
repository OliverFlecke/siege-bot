use std::sync::Arc;

use async_trait::async_trait;
use serenity::{
    http::Http,
    model::user::User,
    prelude::{Context, RwLock, TypeMap},
};
use uuid::Uuid;

use super::{discord_app_command::DiscordAppCmd, CommandError};

/// Wrapper for the `serenity::Context` for mocking.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiscordContext: Sync + Send {
    /// Access the internal data store for the service.
    fn data(&self) -> &Arc<RwLock<TypeMap>>;

    /// Access the `Http` on the context.
    ///
    /// This is wrapped in an optional to allow ignoring it for testing.
    fn http(&self) -> Option<Arc<Http>>;

    /// Find a Siege player based on the user.
    async fn lookup_siege_player<Cmd>(
        &self,
        command: &Cmd,
        user: &User,
    ) -> Result<Uuid, CommandError>
    where
        Cmd: DiscordAppCmd + 'static;
}

/// Implementation for wrapper trait.
#[async_trait]
impl DiscordContext for Context {
    fn data(&self) -> &Arc<RwLock<TypeMap>> {
        &self.data
    }

    fn http(&self) -> Option<Arc<Http>> {
        Some(self.http.clone())
    }

    async fn lookup_siege_player<Cmd>(
        &self,
        command: &Cmd,
        user: &User,
    ) -> Result<Uuid, CommandError>
    where
        Cmd: DiscordAppCmd + 'static,
    {
        let data = self.data().read().await;
        let lookup = data
            .get::<crate::siege_player_lookup::SiegePlayerLookup>()
            .expect("always registered");
        let lookup = lookup.read().await;

        match lookup.get(&user.id) {
            Some(player_id) => Ok(*player_id),
            None => {
                command.send_text(
                    self.http(),
                    format!(
                        "No Siege player found for {}.\nUse the `/add` command to link your Discord profile to your Ubisoft name",
                        user.tag()
                    ).as_str(),
                )
                .await?;
                Err(CommandError::SiegePlayerNotFound)
            }
        }
    }
}
