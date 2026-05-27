use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    impl_command_execute_2, impl_command_io,
    menus::select_period::select_period,
    storages::{ExpensePeriod, Storage},
    ui::{ButtonData, CommandContext},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandClearExpenses {
    pub period: Option<ExpensePeriod>,
    pub confirm: Option<bool>,
}

impl CommandClearExpenses {
    async fn run0(&self, target: &CommandContext, storage: Arc<Storage>) -> ResponseResult<()> {
        let var_storage = storage.variables();
        let current_period: Option<ExpensePeriod> = var_storage.get().await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        let prompt = markdown_format!(
            "🗑️ *Clear expenses for period*\n\n\
             Current period: *{}*\n\n\
             Select a period to clear its expenses:",
            current_period_str
        );

        let expense_storage = storage.expenses_readonly();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| {
                CommandClearExpenses {
                    period: Some(*period),
                    confirm: None,
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
        _storage: Arc<Storage>,
        period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        let message = markdown_format!(
            "🗑️ Confirm clearing all expenses for period *{}*\\?",
            period.to_string()
        );

        let buttons = vec![vec![ButtonData::SwitchInlineQuery(
            "✅ Yes, Clear All".to_string(),
            CommandClearExpenses {
                period: Some(*period),
                confirm: Some(true),
            }
            .to_command_string(false),
        )]];

        target.markdown_message_with_menu(message, buttons).await?;
        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        period: &ExpensePeriod,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_string!("❌ Clear expenses cancelled\\."))
                .await?;
            return Ok(());
        }

        storage.expenses().clear_expenses(*period).await;

        target
            .send_markdown_message(markdown_format!(
                "🗑️ All expenses for period *{}* cleared\\!",
                period.to_string()
            ))
            .await?;
        Ok(())
    }
}

impl_command_io!(
    CommandClearExpenses,
    "clear_expenses",
    ["period", "confirm"],
    period: ExpensePeriod,
    confirm: bool
);
impl_command_execute_2!(CommandClearExpenses, Arc<Storage>, period, confirm);

impl From<CommandClearExpenses> for crate::commands::Command {
    fn from(cmd: CommandClearExpenses) -> Self {
        crate::commands::Command::ClearExpenses(cmd)
    }
}
