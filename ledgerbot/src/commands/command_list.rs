use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand}, markdown_string
};

use crate::{
    commands::
        expenses::format_expenses_chronological
    ,
    menus::{common::show_follow_status_message, select_period::select_period},
    storages::{ExpensePeriod, StorageReadonly},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandList {
    pub period: Option<ExpensePeriod>,
}

impl CommandTrait for CommandList {
    type A = ExpensePeriod;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<StorageReadonly>;

    const NAME: &'static str = "list";
    const PLACEHOLDERS: &[&'static str] = &["period"];

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
        CommandList { period }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        show_follow_status_message(target, &storage).await?;

        let prompt = markdown_string!(
            "📋 *List expenses for period*\n\n\
             Select a period to view its expenses:"
        );

        // Show menu with available periods
        let expense_storage = storage.expenses_readonly();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| {
                // Parse the period string from the menu
                CommandList {
                    period: Some(*period),
                }
            },
            None::<CommandList>,
            None::<NoopCommand>, // No new period button for list, only existing periods
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        show_follow_status_message(target, &storage).await?;

        let chat_expenses = storage
            .expenses_readonly()
            .get_expenses(*period)
            .await;

        match format_expenses_chronological(&chat_expenses) {
            Ok(messages) => {
                // List of expenses - send each message
                for message in messages {
                    target.send_markdown_message(message).await?;
                }
            }
            Err(error_message) => {
                // Error message (e.g., no expenses) - send as MarkdownString
                target.send_markdown_message(error_message).await?;
            }
        }

        Ok(())
    }
}

impl From<CommandList> for crate::commands::Command {
    fn from(cmd: CommandList) -> Self {
        crate::commands::Command::List(cmd)
    }
}
