use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg};

use crate::{
    commands::report::{check_category_conflicts, format_category_summary, format_single_category_report},
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
            target.markdown_message(conflict_message).await?;
            return Ok(());
        }

        // Show summary with category selection menu
        let (message, buttons) = format_category_summary(&chat_expenses, &chat_categories);

        if buttons.is_empty() {
            // No categories, just send the message
            target.markdown_message(message).await?;
        } else {
            // Send message with category selection menu
            target.markdown_message_with_menu(message, buttons).await?;
        }

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &Self::A,
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
            target.markdown_message(conflict_message).await?;
            return Ok(());
        }

        // Filter expenses for the requested category
        let category_expenses: Vec<_> = if category == "Other" {
            // "Other" category: uncategorized expenses
            let category_matchers: Vec<(String, Vec<regex::Regex>)> = chat_categories
                .iter()
                .map(|(name, patterns)| {
                    let regexes: Vec<regex::Regex> = patterns
                        .iter()
                        .filter_map(|pattern| regex::Regex::new(pattern).ok())
                        .collect();
                    (name.clone(), regexes)
                })
                .collect();

            chat_expenses
                .iter()
                .filter(|expense| {
                    // Check if expense doesn't match any category
                    !category_matchers.iter().any(|(_, regexes)| {
                        regexes.iter().any(|re| re.is_match(&expense.description))
                    })
                })
                .cloned()
                .collect()
        } else {
            // Specific category: expenses matching this category's filters
            let patterns = chat_categories.get(category);
            if let Some(patterns) = patterns {
                let regexes: Vec<regex::Regex> = patterns
                    .iter()
                    .filter_map(|pattern| regex::Regex::new(pattern).ok())
                    .collect();

                chat_expenses
                    .iter()
                    .filter(|expense| regexes.iter().any(|re| re.is_match(&expense.description)))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Format the report using line-by-line approach with overflow detection
        let message = format_single_category_report(category, &category_expenses);

        // Add a "Back" button to return to summary view
        let back_button = vec![vec![yoroolbot::storage::ButtonData::Callback(
            "↩️ Back to Summary".to_string(),
            CommandReport { category: None }.to_command_string(false),
        )]];

        target.markdown_message_with_menu(message, back_button).await?;

        Ok(())
    }
}

impl From<CommandReport> for crate::commands::Command {
    fn from(cmd: CommandReport) -> Self {
        crate::commands::Command::Report(cmd)
    }
}
