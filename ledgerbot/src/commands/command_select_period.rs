use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::prelude::ResponseResult;

use crate::{
    commands::Command,
    impl_command_execute_1, impl_command_io,
    menus::select_period::select_period,
    storages::{ExpensePeriod, Storage},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandSelectPeriod {
    pub period: Option<ExpensePeriod>,
}

impl_command_io!(CommandSelectPeriod, "select_period", ["YYYY-MM"], period: ExpensePeriod);
impl_command_execute_1!(CommandSelectPeriod, Arc<Storage>, period);

impl CommandSelectPeriod {
    async fn run0(&self, target: &CommandContext, storage: Arc<Storage>) -> ResponseResult<()> {
        let var_storage = storage.variables();
        let current_period: Option<ExpensePeriod> = var_storage.get().await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        let new_period_command = CommandSelectPeriod { period: None };
        let new_period_command_str = new_period_command.to_command_string(true);

        let prompt = markdown_format!(
            "📅 Current period: *{}*\n\n\
             Select a period from the list below, use the ➕ button for a new period, or use `{}` to enter manually\\.",
            current_period_str,
            new_period_command_str
        );

        let expense_storage = storage.expenses_readonly();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| {
                Command::SelectPeriod(CommandSelectPeriod {
                    period: Some(*period),
                })
            },
            None,
            Some(Command::SelectPeriod(new_period_command)),
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        let var_storage = storage.variables();
        var_storage.set(*period).await;

        target
            .send_markdown_message(markdown_format!(
                "📅 Period selected: *{}*\n\n\
                 All expense operations will now use this period\\.",
                period.to_string()
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
