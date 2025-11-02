use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{
    menus::select_period::select_period,
    storages::{ExpensePeriod, StorageTrait},
};

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

    type Context = Arc<dyn StorageTrait>;

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
        let var_storage = storage.clone().as_variable_storage();
        let current_period: Option<ExpensePeriod> = var_storage.get(chat_id).await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        let prompt = markdown_format!(
            "📅 Current period: *{}*\n\n\
             Select a period from the list below or use `/select\\_period <YYYY\\-MM>` to enter manually\\.",
            current_period_str
        );

        // Show menu with available periods
        let expense_storage = storage.clone().as_expense_storage();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| CommandSelectPeriod {
                period: Some(period.to_string()),
            },
            None::<CommandSelectPeriod>,
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period_str: &String,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;

        // Parse the period string (format: YYYY-MM)
        let period = match ExpensePeriod::from_string(period_str) {
            Ok(p) => p,
            Err(err) => {
                target
                    .send_markdown_message(markdown_format!(
                        "❌ Invalid period format: {}\n\n\
                         Please use format YYYY\\-MM \\(e\\.g\\., 2024\\-03\\)",
                        err
                    ))
                    .await?;
                return Ok(());
            }
        };

        // Store the selected period in VariableStorage
        let var_storage = storage.clone().as_variable_storage();
        var_storage.set(chat_id, period).await;

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
