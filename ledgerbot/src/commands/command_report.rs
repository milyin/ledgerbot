use std::sync::Arc;

use teloxide::prelude::ResponseResult;

use crate::{
    commands::{
        command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
        report::{check_category_conflicts, format_expenses_list},
    },
    storage_traits::StorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandReport;

impl CommandTrait for CommandReport {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "report";
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
        CommandReport
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let chat_expenses = storage
            .clone()
            .as_expense_storage()
            .get_chat_expenses(chat_id)
            .await;
        let chat_categories = storage
            .clone()
            .as_category_storage()
            .get_chat_categories(chat_id)
            .await;

        // Check for category conflicts before generating report
        if let Some(conflict_message) = check_category_conflicts(&chat_expenses, &chat_categories) {
            target.send_markdown_message(conflict_message).await?;
            return Ok(());
        }

        let expenses_list = format_expenses_list(&chat_expenses, &chat_categories);

        target.send_markdown_message(expenses_list).await?;
        Ok(())
    }
}

impl From<CommandReport> for crate::commands::Command {
    fn from(cmd: CommandReport) -> Self {
        crate::commands::Command::Report(cmd)
    }
}
