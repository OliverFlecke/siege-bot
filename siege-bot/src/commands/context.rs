use std::sync::Arc;

use async_trait::async_trait;
use serenity::{
    http::Http,
    model::user::User,
    prelude::{Context, RwLock, TypeMap},
};
use uuid::Uuid;

use super::{discord_app_command::DiscordAppCmd, CmdResult, CommandError};

/// Wrapper for the `serenity::Context` for mocking.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DiscordContext: Sync + Send {
    /// Send a simple text message through the context.
    async fn send_text_message<C>(&self, command: &C, content: &str) -> CmdResult
    where
        C: DiscordAppCmd + 'static;

    /// Access the internal data store for the service.
    fn data(&self) -> &Arc<RwLock<TypeMap>>;

    fn http(&self) -> &Arc<Http>;

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
    async fn send_text_message<C>(&self, command: &C, content: &str) -> CmdResult
    where
        C: DiscordAppCmd,
    {
        command.send_text(self.http.clone(), content).await
    }

    fn data(&self) -> &Arc<RwLock<TypeMap>> {
        &self.data
    }

    fn http(&self) -> &Arc<Http> {
        &self.http
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
                self.send_text_message(
                    command,
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
