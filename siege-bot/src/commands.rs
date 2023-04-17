use async_trait::async_trait;
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};
use thiserror::Error;

use self::{
    context::DiscordContext,
    discord_app_command::{DiscordAppCmd, DiscordAutocompleteInteraction},
};

pub mod add_player;
pub mod all_maps;
pub mod all_operators;
pub mod context;
pub mod discord_app_command;
pub mod id;
pub mod map;
pub mod operator;
pub mod ping;
pub mod statistics;

#[async_trait]
pub trait CommandHandler {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync;
}

#[async_trait]
pub trait AutocompleteHandler {
    async fn handle_autocomplete<Ctx, Cmd>(ctx: &Ctx, cmd: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAutocompleteInteraction + Send + Sync;
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command does not exists")]
    CommandNotFound,
    #[error("internal Discord error")]
    SerenityError(serenity::Error),
    #[error("Siege player not found")]
    SiegePlayerNotFound,
}

pub type CmdResult = core::result::Result<(), CommandError>;

trait AddUserOptionToCommand {
    fn add_user_option(&mut self) -> &mut Self;
}

impl AddUserOptionToCommand for CreateApplicationCommand {
    fn add_user_option(&mut self) -> &mut CreateApplicationCommand {
        self.create_option(|option| {
            option
                .name("user")
                .description("The user to get statistics for. Defaults to the sending user")
                .kind(CommandOptionType::User)
                .required(false)
        })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::sync::Arc;

    use async_trait::async_trait;
    use serenity::prelude::{RwLock, TypeMap};
    use siege_api::models::{FullProfile, PlaytimeProfile, StatisticResponse};
    use uuid::Uuid;

    use crate::SiegeApi;

    use super::context::MockDiscordContext;

    mockall::mock! {
        pub SiegeClient {}

        #[allow(implied_bounds_entailment)]
        #[async_trait]
        impl siege_api::client::SiegeClient for SiegeClient {
            async fn search_for_player(&self, name: &str) -> siege_api::client::Result<Uuid>;
            async fn get_playtime(&self, player_id: Uuid) -> siege_api::client::Result<PlaytimeProfile>;
            async fn get_full_profiles(&self, player_id: Uuid) -> siege_api::client::Result<Vec<FullProfile>>;
            async fn get_operators(&self, player_id: Uuid) -> siege_api::client::Result<StatisticResponse>;
            async fn get_maps(&self, player_id: Uuid) -> siege_api::client::Result<StatisticResponse>;
        }
    }

    // rust-analyzer is throwing an error here, but this still compiles fine.
    // Looks like its an issue with importing these mocks from an external crate.
    // I have not found a way to ignore this error yet, but have instead created a
    // central function to create this mock.
    // use siege_api::client::MockSiegeClient;

    pub fn create_mock_siege_client() -> MockSiegeClient {
        MockSiegeClient::new()
    }

    pub async fn register_client_in_type_map(
        ctx: &mut MockDiscordContext,
        client: MockSiegeClient,
    ) {
        let data = Arc::new(RwLock::new(TypeMap::default()));
        {
            let mut data = data.write().await;
            data.insert::<SiegeApi>(Arc::new(client));
        }
        ctx.expect_data().return_const(data);
    }
}
