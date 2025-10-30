use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_string,
    storage::ButtonData,
};

use crate::storages::storage_traits::ExpenseStorageTrait;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandClearExpenses {
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandClearExpenses {
    type A = bool;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn ExpenseStorageTrait>;

    const NAME: &'static str = "clear_expenses";
    const PLACEHOLDERS: &[&'static str] = &["<confirm>"];

    fn param1(&self) -> Option<&Self::A> {
        self.confirm.as_ref()
    }

    fn from_arguments(
        confirm: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandClearExpenses { confirm }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _storage: Self::Context,
    ) -> ResponseResult<()> {
        // Show confirmation prompt with buttons
        let message = markdown_string!("🗑️ Confirm clearing all expenses\\?");

        let buttons = vec![vec![ButtonData::SwitchInlineQuery(
            "✅ Yes, Clear All".to_string(),
            CommandClearExpenses {
                confirm: Some(true),
            }
            .to_command_string(false),
        )]];

        target.markdown_message_with_menu(message, buttons).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_string!("❌ Clear expenses cancelled\\."))
                .await?;
            return Ok(());
        }

        let chat_id = target.chat.id;
        storage.clear_chat_expenses(chat_id).await;

        target
            .send_markdown_message(markdown_string!("🗑️ All expenses cleared\\!"))
            .await?;
        Ok(())
    }
}

impl From<CommandClearExpenses> for crate::commands::Command {
    fn from(cmd: CommandClearExpenses) -> Self {
        crate::commands::Command::ClearExpenses(cmd)
    }
}
