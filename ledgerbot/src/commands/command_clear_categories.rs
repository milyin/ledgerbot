use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::markdown_string;

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandClearCategories;

impl CommandTrait for CommandClearCategories {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "clear_categories";
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
        CommandClearCategories
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        storage.clear_chat_categories(chat_id).await;

        target
            .send_markdown_message(markdown_string!("🗑️ All categories cleared\\!"))
            .await?;
        Ok(())
    }
}

impl From<CommandClearCategories> for crate::commands::Command {
    fn from(cmd: CommandClearCategories) -> Self {
        crate::commands::Command::ClearCategories(cmd)
    }
}
