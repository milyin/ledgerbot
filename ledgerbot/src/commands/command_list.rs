use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::markdown_format;

use crate::commands::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    expenses::format_expenses_chronological,
};
use crate::storage_traits::ExpenseStorageTrait;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandList;

impl CommandTrait for CommandList {
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

    const NAME: &'static str = "list";
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
        CommandList
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let chat_expenses = storage.get_chat_expenses(chat_id).await;
        let expenses_list = format_expenses_chronological(&chat_expenses);

        target
            .send_markdown_message(markdown_format!("{}", expenses_list))
            .await?;
        Ok(())
    }
}

impl From<CommandList> for crate::commands::Command {
    fn from(cmd: CommandList) -> Self {
        crate::commands::Command::List(cmd)
    }
}
