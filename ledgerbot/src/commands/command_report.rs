use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg};

use crate::{
    commands::report::{check_category_conflicts, format_expenses_by_category, format_category_summary},
    storage_traits::StorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandReport {
    pub category: Option<String>,
}

impl CommandTrait for CommandReport {
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

    const NAME: &'static str = "report";
    const PLACEHOLDERS: &[&'static str] = &["category"];

    fn from_arguments(
        category: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandReport { category }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
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
            .await
            .unwrap_or_default();

        // Check for category conflicts before generating report
        if let Some(conflict_message) = check_category_conflicts(&chat_expenses, &chat_categories) {
            target.send_markdown_message(conflict_message).await?;
            return Ok(());
        }

        // If category is specified, show detailed report for that category only
        if let Some(ref category_name) = self.category {
            let messages = format_expenses_by_category(&chat_expenses, &chat_categories);

            // Find the message for the requested category
            // The format_expenses_by_category returns messages in order: categories (sorted), Other, Total
            // We need to find the one matching our category
            let mut found = false;
            for message in messages {
                let msg_str = message.as_str();
                // Check if this message starts with the category name
                if msg_str.starts_with(&format!("*{}*:", category_name)) {
                    target.send_markdown_message(message).await?;
                    found = true;
                    break;
                }
            }

            if !found {
                target.send_markdown_message(
                    yoroolbot::markdown_format!("Category '{}' not found or has no expenses\\.", category_name)
                ).await?;
            }
        } else {
            // No category specified - show summary with category selection menu
            let (message, buttons) = format_category_summary(&chat_expenses, &chat_categories);

            if buttons.is_empty() {
                // No categories, just send the message
                target.send_markdown_message(message).await?;
            } else {
                // Send message with category selection menu
                target.send_markdown_message_with_menu(message, buttons).await?;
            }
        }

        Ok(())
    }
}

impl From<CommandReport> for crate::commands::Command {
    fn from(cmd: CommandReport) -> Self {
        crate::commands::Command::Report(cmd)
    }
}
