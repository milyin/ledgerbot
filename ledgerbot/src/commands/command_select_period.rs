use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_string,
};

use crate::storages::ExpenseStorageTrait;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandSelectPeriod {
    pub period: Option<String>,
}

impl CommandTrait for CommandSelectPeriod {
    type A = String;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn ExpenseStorageTrait>;

    const NAME: &'static str = "select_period";
    const PLACEHOLDERS: &[&'static str] = &["<period_name>"];

    fn param1(&self) -> Option<&Self::A> {
        self.period.as_ref()
    }

    fn from_arguments(
        period: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandSelectPeriod { period }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let current_period = storage.get_selected_period(chat_id).await;

        let message = if let Some(period) = current_period {
            yoroolbot::markdown_format!(
                "📅 Current period: *{}*\n\nUse `/select\\_period <name>` to switch periods\\.",
                period
            )
        } else {
            markdown_string!("📅 No period selected \\(using default\\)\\.\n\nUse `/select\\_period <name>` to select a period\\.")
        };

        target.send_markdown_message(message).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &String,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        storage.select_period(chat_id, period.clone()).await;

        target
            .send_markdown_message(yoroolbot::markdown_format!(
                "📅 Period selected: *{}*",
                period
            ))
            .await?;
        Ok(())
    }
}

impl From<CommandSelectPeriod> for crate::commands::Command {
    fn from(cmd: CommandSelectPeriod) -> Self {
        crate::commands::Command::SelectPeriod(cmd)
    }
}
