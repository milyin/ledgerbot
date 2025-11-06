use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
    storage::{self, ButtonData},
};

use crate::{
    menus::select_period::select_period,
    storages::{ExpensePeriod, Stores},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandClearExpenses {
    pub period: Option<ExpensePeriod>,
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandClearExpenses {
    type A = ExpensePeriod;
    type B = bool;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<Stores>;

    const NAME: &'static str = "clear_expenses";
    const PLACEHOLDERS: &[&'static str] = &["period", "confirm"];

    fn param1(&self) -> Option<&Self::A> {
        self.period.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.confirm.as_ref()
    }

    fn from_arguments(
        period: Option<Self::A>,
        confirm: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandClearExpenses { period, confirm }
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

        let prompt = markdown_format!(
            "🗑️ *Clear expenses for period*\n\n\
             Current period: *{}*\n\n\
             Select a period to clear its expenses:",
            current_period_str
        );

        // Show menu with available periods
        let storage_ = storage.storage(chat_id);
        let expense_storage = storage_.expenses();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| CommandClearExpenses {
                period: Some(*period),
                confirm: None,
            },
            None::<CommandClearExpenses>,
            None::<NoopCommand>, // No new period button
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        _storage: Self::Context,
        period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        // Show confirmation prompt with buttons
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
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_string!("❌ Clear expenses cancelled\\."))
                .await?;
            return Ok(());
        }

        let chat_id = target.chat.id;

        let storage_ = storage.storage(chat_id);
        storage_
            .expenses()
            .clear_expenses(*period)
            .await;

        target
            .send_markdown_message(markdown_format!(
                "🗑️ All expenses for period *{}* cleared\\!",
                period.to_string()
            ))
            .await?;
        Ok(())
    }
}

impl From<CommandClearExpenses> for crate::commands::Command {
    fn from(cmd: CommandClearExpenses) -> Self {
        crate::commands::Command::ClearExpenses(cmd)
    }
}
