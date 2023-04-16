use async_trait::async_trait;
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};
use thiserror::Error;

use self::{context::DiscordContext, discord_app_command::DiscordAppCmd};

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
    use async_trait::async_trait;
    use siege_api::models::{FullProfile, PlaytimeProfile, StatisticResponse};
    use uuid::Uuid;

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
}
