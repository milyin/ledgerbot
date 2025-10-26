use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::markdown_string;

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    storage_traits::ExpenseStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandClearExpenses;

impl CommandTrait for CommandClearExpenses {
    type A = EmptyArg;
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
    const PLACEHOLDERS: &[&'static str] = &[];

    fn from_arguments(
        _: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandClearExpenses
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
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
        crate::commands::Command::Clear(cmd)
    }
}
