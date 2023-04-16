use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
    },
};

use crate::{siege_player_lookup::SiegePlayerLookup, SiegeApi};

use super::{
    context::DiscordContext, discord_app_command::DiscordAppCmd, CmdResult, CommandHandler,
};

pub struct AddPlayerCommand;

static UBISOFT_NAME: &str = "ubisoft_name";

#[async_trait]
impl CommandHandler for AddPlayerCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("add")
            .description("Link your Ubisoft player ID to your Discord Id. This is required before using most commands")
            .create_option(|option| {
                option
                    .name(UBISOFT_NAME)
                    .description("Name used on your Ubisoft account")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("user")
                    .description("The user to link. Defaults to the sending user")
                    .kind(CommandOptionType::User)
                    .required(false)
            })
    }

    async fn run<Ctx, Cmd>(ctx: &Ctx, command: &Cmd) -> CmdResult
    where
        Ctx: DiscordContext + Send + Sync,
        Cmd: DiscordAppCmd + 'static + Send + Sync,
    {
        let name = match command.get_option(UBISOFT_NAME) {
            Some(CommandDataOptionValue::String(value)) => value,
            _ => unreachable!(),
        };

        let user = command.get_user_from_command_or_default();
        tracing::info!("Linking {} with Ubisoft account {name}", user.tag());

        let ubisoft_id = {
            let data = ctx.data().read().await;
            let siege_client = data.get::<SiegeApi>().expect("client always registered");
            match siege_client.search_for_player(&name).await {
                Ok(id) => id,
                Err(err) => {
                    tracing::error!("Could not find player with that id. Error: {err:?}");
                    return command
                        .send_text(ctx.http(), "No player found with that name")
                        .await;
                }
            }
        };

        {
            let data = ctx.data().write().await;
            let lookup = data.get::<SiegePlayerLookup>().expect("always registered");
            let mut lookup = lookup.write().await;

            match lookup.insert(&user.id, ubisoft_id) {
                Ok(_) => {
                    command.send_text(ctx.http(), "Accounts linked!").await?;
                    // ctx.send_text_message(command, "Accounts linked!").await?;
                }
                Err(err) => {
                    tracing::error!("Failed to store user: {err:?}");
                    return command
                        .send_text(ctx.http(), "Failed to link your accounts")
                        .await;
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use mockall::predicate::*;
    use serenity::{model::user::User, prelude::RwLock};
    use siege_api::auth::ConnectError;
    use uuid::Uuid;

    use crate::{
        commands::{
            context::MockDiscordContext,
            discord_app_command::MockDiscordAppCmd,
            test::{create_mock_siege_client, register_client_in_type_map},
        },
        siege_player_lookup::MockPlayerLookup,
    };

    use super::*;

    #[test]
    fn validate_register() {
        let mut command = CreateApplicationCommand::default();

        // Act
        let command = AddPlayerCommand::register(&mut command);

        // Assert
        assert_eq!(command.0.get("name").unwrap(), "add");
        assert!(!command
            .0
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn validate_run() {
        let user = User::default();
        let siege_id = Uuid::new_v4();
        let ubisoft_name = "some_name".to_string();

        // Ensure the expected message is sent back through the command
        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);

        let mut mock_client = create_mock_siege_client();
        mock_client
            .expect_search_for_player()
            .with(eq(ubisoft_name.clone()))
            .once()
            .return_once(move |_| Ok(siege_id));

        // Ensure the right user/id pair is inserted into the lookup.
        let mut mock_lookup = MockPlayerLookup::default();
        mock_lookup
            .expect_insert()
            .with(eq(user.id), eq(siege_id))
            .once()
            .return_once(|_, _| Ok(()));

        register_client_in_type_map(&mut ctx, mock_client).await;
        {
            let mut data = ctx.data().write().await;
            data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(mock_lookup)));
        }

        // Arrange command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_option()
            .with(eq(UBISOFT_NAME))
            .return_once(move |_| Some(CommandDataOptionValue::String(ubisoft_name)));
        command
            .expect_get_user_from_command_or_default()
            .return_once(|| user);
        command
            .expect_send_text()
            .once()
            .with(always(), eq("Accounts linked!"))
            .return_once(|_, _| Ok(()));

        // Act
        assert!(AddPlayerCommand::run(&ctx, &command).await.is_ok());
    }

    #[tokio::test]
    async fn validate_run_failed_save() {
        let user = User::default();
        let siege_id = Uuid::new_v4();
        let ubisoft_name = "some_name".to_string();

        // Ensure the expected message is sent back through the command
        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);

        let mut mock_client = create_mock_siege_client();
        mock_client
            .expect_search_for_player()
            .with(eq(ubisoft_name.clone()))
            .once()
            .return_once(move |_| Ok(siege_id));

        // Throw an error when trying to save players
        let mut mock_lookup = MockPlayerLookup::default();
        mock_lookup
            .expect_insert()
            .with(eq(user.id), eq(siege_id))
            .once()
            .return_once(|_, _| Err(std::io::Error::new(std::io::ErrorKind::Other, "")));

        register_client_in_type_map(&mut ctx, mock_client).await;
        {
            let mut data = ctx.data().write().await;
            data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(mock_lookup)));
        }

        // Arrange command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_option()
            .with(eq(UBISOFT_NAME))
            .return_once(move |_| Some(CommandDataOptionValue::String(ubisoft_name)));
        command
            .expect_get_user_from_command_or_default()
            .return_once(|| user);

        // Assert the right message is set
        command
            .expect_send_text()
            .once()
            .with(always(), eq("Failed to link your accounts"))
            .returning(|_, _| Ok(()));

        // Act
        assert!(AddPlayerCommand::run(&ctx, &command).await.is_ok());
    }

    #[tokio::test]
    async fn validate_run_player_not_found() {
        let user = User::default();
        let ubisoft_name = "some_name".to_string();

        // Ensure the expected message is sent back through the command
        let mut ctx = MockDiscordContext::new();
        ctx.expect_http().return_const(None);

        let mut mock_client = create_mock_siege_client();
        mock_client
            .expect_search_for_player()
            .with(eq(ubisoft_name.clone()))
            .once()
            .return_once(|_| Err(ConnectError::InvalidPassword));

        let mock_lookup = MockPlayerLookup::default();
        register_client_in_type_map(&mut ctx, mock_client).await;
        {
            let mut data = ctx.data().write().await;
            data.insert::<SiegePlayerLookup>(Arc::new(RwLock::new(mock_lookup)));
        }

        // Arrange command
        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_option()
            .with(eq(UBISOFT_NAME))
            .return_once(move |_| Some(CommandDataOptionValue::String(ubisoft_name)));
        command
            .expect_get_user_from_command_or_default()
            .return_once(|| user);
        // Assert the right message is sent back
        command
            .expect_send_text()
            .once()
            .with(always(), eq("No player found with that name"))
            .returning(|_, _| Ok(()));

        // Act
        assert!(AddPlayerCommand::run(&ctx, &command).await.is_ok());
    }
}
