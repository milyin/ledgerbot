use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format,
};

use crate::{
    commands::expenses::format_expenses_chronological,
    menus::select_period::select_period,
    storages::{ExpensePeriod, StorageTrait},
};

#[derive(Default, Debug, Clone, PartialEq)]
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

    type Context = Arc<dyn StorageTrait>;

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
        let chat_id = target.chat.id;
        let var_storage = storage.clone().as_variable_storage();
        let current_period: Option<ExpensePeriod> = var_storage.get(chat_id).await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        let prompt = markdown_format!(
            "📋 *List expenses for period*\n\n\
             Current period: *{}*\n\n\
             Select a period to view its expenses:",
            current_period_str
        );

        // Show menu with available periods
        let expense_storage = storage.clone().as_expense_storage();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period_str| {
                // Parse the period string from the menu
                match ExpensePeriod::from_string(period_str) {
                    Ok(period) => CommandList {
                        period: Some(period),
                    },
                    Err(_) => CommandList { period: None },
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
        let chat_id = target.chat.id;

        let chat_expenses = storage
            .clone()
            .as_expense_storage()
            .get_expenses(chat_id, period.to_string())
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
