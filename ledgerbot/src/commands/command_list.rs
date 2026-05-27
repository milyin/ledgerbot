use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_string;
use teloxide::prelude::ResponseResult;

use crate::{
    commands::expenses::format_expenses_chronological,
    impl_command_execute_1, impl_command_io,
    menus::{common::show_follow_status_message, select_period::select_period},
    storages::{ExpensePeriod, StorageReadonly},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandList {
    pub period: Option<ExpensePeriod>,
}

impl_command_io!(CommandList, "list", ["period"], period: ExpensePeriod);
impl_command_execute_1!(CommandList, Arc<StorageReadonly>, period);

impl CommandList {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
    ) -> ResponseResult<()> {
        show_follow_status_message(target, &storage).await?;

        let prompt = markdown_string!(
            "📋 *List expenses for period*\n\n\
             Select a period to view its expenses:"
        );

        let expense_storage = storage.expenses_readonly();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| {
                CommandList {
                    period: Some(*period),
                }
                .into()
            },
            None,
            None,
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
        period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        show_follow_status_message(target, &storage).await?;

        let chat_expenses = storage.expenses_readonly().get_expenses(*period).await;

        match format_expenses_chronological(&chat_expenses) {
            Ok(messages) => {
                for message in messages {
                    target.send_markdown_message(message).await?;
                }
            }
            Err(error_message) => {
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
