use std::collections::HashSet;

use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::Timestamp,
    utils::Color,
};
use siege_api::models::meta::Status;

use crate::SiegeApi;

use super::{
    context::DiscordContext, discord_app_command::DiscordAppCmd, CmdResult, CommandHandler,
};

pub struct GameStatusCommand;

#[async_trait]
impl CommandHandler for GameStatusCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("status")
            .description("Get the current game status for Rainbow Six Siege")
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let game_status = {
            let data = ctx.data().read().await;
            let siege_client = data.get::<SiegeApi>().expect("client always registered");
            match siege_client.siege_status().await {
                Ok(id) => id,
                Err(err) => {
                    tracing::error!("{err:?}");
                    return command
                        .send_text(ctx.http(), "Failed to retreive game status")
                        .await;
                }
            }
        };

        command
            .send_embedded(
                ctx.http().clone(),
                CreateEmbed::default()
                    .title("Rainbow Six Siege status")
                    .timestamp(Timestamp::now())
                    .field(
                        "Overall status",
                        format!(
                            "{}",
                            game_status
                                .iter()
                                .find(|x| *x.status() != Status::Online)
                                .map(|x| *x.status())
                                .unwrap_or(Status::Online)
                        ),
                        false,
                    )
                    .field(
                        "Details",
                        game_status
                            .iter()
                            .map(|x| x.name())
                            .fold(String::new(), |acc, next| acc + next + "\n"),
                        true,
                    )
                    .field(
                        "Status",
                        game_status
                            .iter()
                            .map(|x| x.status().to_string())
                            .fold(String::new(), |acc, next| acc + &next + "\n"),
                        true,
                    )
                    .field(
                        "Impacted features",
                        Some(
                            game_status
                                .iter()
                                .flat_map(|x| x.impacted_features())
                                .collect::<HashSet<_>>()
                                .iter()
                                .fold(String::new(), |acc, next| acc + next + "\n"),
                        )
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "None".to_string()),
                        false,
                    )
                    .color(Color::DARK_RED)
                    .clone(),
            )
            .await
    }
}

#[cfg(test)]
mod test {
    use siege_api::models::meta::GameStatus;

    use super::*;

    use crate::commands::{
        context::MockDiscordContext,
        discord_app_command::MockDiscordAppCmd,
        test::{register_client_in_type_map, MockSiegeClient},
    };

    #[test]
    fn validate_register() {
        let mut command = CreateApplicationCommand::default();

        // Act
        let command = GameStatusCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "status");
        assert!(!command
            .0
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);

        let mut mock_client = MockSiegeClient::default();
        mock_client.expect_siege_status().once().returning(|| {
            let content = std::fs::read_to_string("../samples/game-status.json").unwrap();
            let statuses: Vec<GameStatus> = serde_json::from_str(content.as_str()).unwrap();
            Ok(statuses
                .into_iter()
                .filter(|x| x.name().starts_with("Rainbow Six Siege"))
                .collect::<Vec<GameStatus>>())
        });
        register_client_in_type_map(&mut ctx, mock_client).await;

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_send_embedded()
            .once()
            .returning(|_, _| Ok(()));

        // Act
        assert!(GameStatusCommand::run(&ctx, &command).await.is_ok());
    }
}
