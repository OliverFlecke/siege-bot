use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
    },
    prelude::Context,
};

use super::{
    context::{DiscordAppCmd, DiscordContext},
    CmdResult, CommandHandler,
};

pub struct IdCommand;

#[async_trait]
impl CommandHandler for IdCommand {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("id")
            .description("Get a user id")
            .create_option(|option| {
                option
                    .name("id")
                    .description("The user to lookup")
                    .kind(CommandOptionType::User)
                    .required(true)
            })
    }

    async fn run(
        _ctx: &Context,
        _command: &ApplicationCommandInteraction,
    ) -> Result<(), super::CommandError> {
        unimplemented!()
    }
}

impl IdCommand {
    pub async fn alternative<C>(ctx: &impl DiscordContext, command: &C) -> CmdResult
    where
        C: DiscordAppCmd + 'static,
    {
        let content = match command.get_option("id") {
            Some(CommandDataOptionValue::User(user, _)) => {
                format!("{}'s id is {}", user.tag(), user.id)
            }
            _ => "Please provide a valid user".to_string(),
        };

        ctx.send_text_message(command, content.as_str()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use mockall::predicate::{self, *};
    use serde_json::{json, Value};
    use serenity::model::user::User;

    use crate::commands::context::{MockDiscordAppCmd, MockDiscordContext};

    use super::*;

    #[test]
    fn validate_register() {
        let mut command = CreateApplicationCommand::default();
        let command = IdCommand::register(&mut command);

        assert_eq!(command.0.get("name").unwrap(), "id");
        assert_eq!(command.0.get("description").unwrap(), "Get a user id");

        let option = command
            .0
            .get("options")
            .unwrap()
            .as_array()
            .unwrap()
            .get(0)
            .unwrap()
            .as_object()
            .unwrap();

        assert_eq!(option.get("description").unwrap(), "The user to lookup");
        assert_eq!(option.get("name").unwrap(), "id");
        assert_eq!(*option.get("required").unwrap(), Value::Bool(true));
        assert_eq!(*option.get("type").unwrap(), json!(6));
    }

    #[tokio::test]
    async fn validate_run() {
        let user = User::default();
        let mut ctx = MockDiscordContext::new();
        ctx.expect_send_text_message::<MockDiscordAppCmd>()
            .once()
            .with(
                predicate::always(),
                eq(format!("{}'s id is {}", user.tag(), user.id)),
            )
            .returning(|_, _| Ok(()));

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_option()
            .with(eq("id"))
            .return_once(move |_| Some(CommandDataOptionValue::User(user.clone(), None)));

        // Act and assert
        assert!(IdCommand::alternative(&ctx, &command).await.is_ok());
    }

    #[tokio::test]
    async fn validate_run_with_missing_user() {
        let mut ctx = MockDiscordContext::new();
        ctx.expect_send_text_message::<MockDiscordAppCmd>()
            .once()
            .with(predicate::always(), eq("Please provide a valid user"))
            .returning(|_, _| Ok(()));

        let mut command = MockDiscordAppCmd::new();
        command
            .expect_get_option()
            .with(eq("id"))
            .returning(|_| None);

        // Act and assert
        assert!(IdCommand::alternative(&ctx, &command).await.is_ok());
    }
}
