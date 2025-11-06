use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{
    menus::select_period::select_period,
    storages::{ExpensePeriod, Stores},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandSelectPeriod {
    pub period: Option<ExpensePeriod>,
}

impl CommandTrait for CommandSelectPeriod {
    type A = ExpensePeriod;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<Stores>;

    const NAME: &'static str = "select_period";
    const PLACEHOLDERS: &[&'static str] = &["YYYY-MM"];

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
        let var_storage = storage.clone().variable_storage();
        let current_period: Option<ExpensePeriod> = var_storage.get(chat_id).await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        // Create inline command for new period with current period as default
        let new_period_command = CommandSelectPeriod { period: None };
        let new_period_command_str = new_period_command.to_command_string(true);

        let prompt = markdown_format!(
            "📅 Current period: *{}*\n\n\
             Select a period from the list below, use the ➕ button for a new period, or use `{}` to enter manually\\.",
            current_period_str,
            new_period_command_str
        );

        // Show menu with available periods
        let storage_ = storage.storage(chat_id);
        let expense_storage = storage_.expenses();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| {
                // Parse the period string from the menu
                CommandSelectPeriod {
                    period: Some(*period),
                }
            },
            None::<CommandSelectPeriod>,
            Some(new_period_command),
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
        let chat_id = target.chat.id;

        // Store the selected period in VariableStorage
        let var_storage = storage.clone().variable_storage();
        var_storage.set(chat_id, *period).await;

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
