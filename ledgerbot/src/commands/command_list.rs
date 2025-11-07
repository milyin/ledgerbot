use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, storage,
};

use crate::{
    commands::{
        expenses::format_expenses_chronological, follow_helper::validate_and_get_follow_access,
    },
    menus::select_period::select_period,
    storages::{ExpensePeriod, Stores},
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

    type Context = Arc<Stores>;

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
        // Validate follow access and get effective chat ID
        let follow_access = match validate_and_get_follow_access(target, &storage).await {
            Ok(access) => access,
            Err(warning_msg) => {
                target.send_markdown_message(warning_msg).await?;
                // Continue with current chat
                crate::commands::follow_helper::FollowAccess {
                    effective_chat_id: target.chat.id,
                    header_note: None,
                }
            }
        };

        let chat_id = follow_access.effective_chat_id;

        // Show follow status as separate message if applicable
        if let Some(header) = follow_access.header_note {
            target.send_markdown_message(header).await?;
        }

        let var_storage = storage.clone().variable_storage();
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
        let storage_ = storage.storage(chat_id);
        let expense_storage = storage_.expenses();
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
        // Validate follow access and get effective chat ID
        let follow_access = match validate_and_get_follow_access(target, &storage).await {
            Ok(access) => access,
            Err(warning_msg) => {
                target.send_markdown_message(warning_msg).await?;
                // Continue with current chat
                crate::commands::follow_helper::FollowAccess {
                    effective_chat_id: target.chat.id,
                    header_note: None,
                }
            }
        };

        let chat_id = follow_access.effective_chat_id;

        // Show follow status as separate message if applicable
        if let Some(header) = follow_access.header_note {
            target.send_markdown_message(header).await?;
        }

        let storage_ = storage.storage(chat_id);
        let chat_expenses = storage_
            .expenses()
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
