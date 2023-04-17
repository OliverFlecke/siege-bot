use async_trait::async_trait;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::command::CommandOptionType,
    utils::Color,
};
use siege_api::operator::Operator;

use crate::{
    constants::{AUTOCOMPLETE_LIMIT, NAME},
    formatting::FormatEmbedded,
    SiegeApi,
};

use super::{
    context::DiscordContext,
    discord_app_command::{DiscordAppCmd, DiscordAutocompleteInteraction},
    AddUserOptionToCommand, AutocompleteHandler, CmdResult, CommandHandler,
};

pub struct OperatorCommand;

#[async_trait]
impl CommandHandler for OperatorCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("operator")
            .description("Get detailed statistics about an operator")
            .create_option(|option| {
                option
                    .name(NAME)
                    .description("Name of the operator")
                    .kind(CommandOptionType::String)
                    .set_autocomplete(true)
                    .required(true)
            })
            .add_user_option()
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let operator = command
            .extract_enum_option(NAME)
            .expect("required argument");
        let user = command.get_user_from_command_or_default();
        let player_id = ctx.lookup_siege_player(command, &user).await?;

        tracing::info!(
            "Getting statistics for operator '{operator}' for {}",
            user.name
        );

        let response = {
            let data = ctx.data().read().await;
            let siege_client = data
                .get::<SiegeApi>()
                .expect("Siege client is always registered");
            match siege_client.get_operators(player_id).await {
                Ok(data) => data,
                Err(err) => {
                    tracing::error!("Failed to fetch data: {err:?}");
                    return command.send_text(ctx.http(), "Failed to fetch data").await;
                }
            }
        };

        let operator = match response.get_operator(operator) {
            Some(operator) => operator,
            None => {
                command
                    .send_text(
                        ctx.http(),
                        format!("{user} has not played as {operator}", user = user.tag()).as_str(),
                    )
                    .await?;

                return Ok(());
            }
        };

        command
            .send_embedded(
                ctx.http().clone(),
                CreateEmbed::default()
                    .thumbnail(operator.name().avatar_url())
                    .title(format!("Operator statistics for {}", operator.name()))
                    .color(Color::BLUE)
                    .format(operator.statistics())
                    .clone(),
            )
            .await
    }
}

#[async_trait]
impl AutocompleteHandler for OperatorCommand {
    /// Handle auto complete for operator names.
    async fn handle_autocomplete<Ctx, Cmd>(ctx: &Ctx, cmd: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAutocompleteInteraction + Send + Sync,
    {
        if let Some(value) = cmd.get_user_input() {
            cmd.create_autocomplete_response(ctx.http(), move |response| {
                use strum::IntoEnumIterator;

                Operator::iter()
                    .map(|op| op.to_string())
                    .filter(|op| op.starts_with(value.as_str()))
                    .take(AUTOCOMPLETE_LIMIT)
                    .for_each(|op| {
                        response.add_string_choice(op.as_str(), op.as_str());
                    });

                response
            })
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use mockall::predicate::*;
    use serde_json::Value;
    use serenity::model::user::User;
    use siege_api::models::StatisticResponse;
    use uuid::Uuid;

    use crate::commands::{
        context::MockDiscordContext,
        discord_app_command::{MockDiscordAppCmd, MockDiscordAutocompleteInteraction},
        test::{register_client_in_type_map, MockSiegeClient},
    };

    use super::*;

    #[tokio::test]
    async fn validate_autocomplete() {
        let mut ctx = MockDiscordContext::default();
        ctx.expect_http().return_const(None);

        let mut cmd = MockDiscordAutocompleteInteraction::default();
        cmd.expect_get_user_input()
            .return_const(Some("Yi".to_string()));

        cmd.expect_create_autocomplete_response()
            .once()
            .return_once(|_, _| Ok(()));

        assert!(OperatorCommand::handle_autocomplete(&ctx, &cmd)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = OperatorCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "operator");
        assert!(!command
            .0
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap()
            .is_empty());

        let options = command.0.get("options").unwrap().as_array().unwrap();
        // Assert first options
        let opt = options.get(0).unwrap();
        assert_eq!(opt.get("name").unwrap(), NAME);
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(true));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 3); // Corresponds to `CommandOptionType::User`
        assert_eq!(*opt.get("autocomplete").unwrap(), Value::Bool(true));

        let opt = options.get(1).unwrap();
        assert_eq!(opt.get("name").unwrap(), "user");
        assert_eq!(*opt.get("required").unwrap(), Value::Bool(false));
        assert_eq!(opt.get("type").unwrap().as_u64().unwrap(), 6); // Corresponds to `CommandOptionType::User`
        assert!(!opt.get("description").unwrap().as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);
        ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
            .with(always(), eq(user.clone()))
            .once()
            .returning(move |_, _| Ok(siege_id));

        let mut mock_client = MockSiegeClient::default();
        mock_client.expect_get_operators().once().returning(|_| {
            let content = std::fs::read_to_string("../samples/operators.json").unwrap();
            let stats: StatisticResponse = serde_json::from_str(content.as_str()).unwrap();
            Ok(stats)
        });
        register_client_in_type_map(&mut ctx, mock_client).await;

        // Setup command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_user_from_command_or_default()
            .return_const(user.clone());
        command
            .expect_extract_enum_option()
            .once()
            .with(eq(NAME))
            .return_const(Operator::Ying);
        command
            .expect_send_embedded()
            .once()
            .with(always(), always())
            .return_once(|_, _| Ok(()));

        assert!(OperatorCommand::run(&ctx, &command).await.is_ok());
    }

    #[tokio::test]
    async fn validate_run_api_failed() {
        let user = User::default();
        let siege_id = Uuid::new_v4();

        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);
        ctx.expect_lookup_siege_player::<MockDiscordAppCmd>()
            .with(always(), eq(user.clone()))
            .once()
            .returning(move |_, _| Ok(siege_id));

        let mut mock_client = MockSiegeClient::default();
        mock_client
            .expect_get_operators()
            .once()
            .returning(|_| Err(siege_api::auth::ConnectError::InvalidPassword));
        register_client_in_type_map(&mut ctx, mock_client).await;

        // Setup command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_user_from_command_or_default()
            .return_const(user.clone());
        command
            .expect_extract_enum_option()
            .once()
            .with(eq(NAME))
            .return_const(Operator::Ying);
        command
            .expect_send_text()
            .once()
            .with(always(), eq("Failed to fetch data"))
            .return_once(|_, _| Ok(()));

        assert!(OperatorCommand::run(&ctx, &command).await.is_ok());
    }
}
