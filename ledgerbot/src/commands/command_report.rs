use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format,
};

use crate::{
    commands::report::{
        check_category_conflicts, filter_category_expenses, format_category_summary,
        format_single_category_report,
    },
    menus::select_period::select_period,
    storages::{Category, ExpensePeriod, StorageTrait},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandReport {
    pub period: Option<ExpensePeriod>,
    pub category: Option<Category>,
    pub page: Option<usize>,
}

impl CommandTrait for CommandReport {
    type A = ExpensePeriod;
    type B = Category;
    type C = usize;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "report";
    const PLACEHOLDERS: &[&'static str] = &["period", "category", "page"];

    fn from_arguments(
        period: Option<Self::A>,
        category: Option<Self::B>,
        page: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandReport {
            period,
            category,
            page,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.period.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.category.as_ref()
    }

    fn param3(&self) -> Option<&Self::C> {
        self.page.as_ref()
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
            "📊 *Report for period*\n\n\
             Current period: *{}*\n\n\
             Select a period to view its report:",
            current_period_str
        );

        // Show menu with available periods
        let expense_storage = storage.clone().as_expense_storage();
        select_period(
            target,
            &expense_storage,
            prompt,
            |period| CommandReport {
                period: Some(*period),
                category: None,
                page: None,
            },
            None::<CommandReport>,
            None::<NoopCommand>, // No new period button for reports, only existing periods
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
            .get_expenses(chat_id, *period)
            .await;
        let chat_categories = storage
            .clone()
            .as_category_storage()
            .get_chat_categories(chat_id)
            .await
            .unwrap_or_default();

        let all_expenses = storage
            .clone()
            .as_expense_storage()
            .get_all_expenses(chat_id)
            .await
            .into_iter()
            .map(|(_, expense)| expense)
            .collect::<Vec<_>>();

        if let Some(conflict_message) = check_category_conflicts(&all_expenses, &chat_categories) {
            target.markdown_message(conflict_message).await?;
            return Ok(());
        }

        // Create back button to return to period selection
        let back_button = Some(yoroolbot::storage::ButtonData::Callback(
            "↩️ Back to Periods".to_string(),
            CommandReport {
                period: None,
                category: None,
                page: None,
            }
            .to_command_string(false),
        ));

        // Show summary with category selection menu
        let (message, buttons) = format_category_summary(
            &chat_expenses,
            &chat_categories,
            &period.to_string(),
            back_button,
        );

        if buttons.is_empty() {
            // No categories, just send the message
            target.markdown_message(message).await?;
        } else {
            // Send message with category selection menu
            target.markdown_message_with_menu(message, buttons).await?;
        }

        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
        category: &Category,
    ) -> ResponseResult<()> {
        // Default to page 0 if not specified
        self.run3(target, storage, period, category, &0).await
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
        category: &Category,
        page: &usize,
    ) -> ResponseResult<()> {
        const RECORDS_PER_PAGE: usize = 25;

        let chat_id = target.chat.id;

        let chat_expenses = storage
            .clone()
            .as_expense_storage()
            .get_expenses(chat_id, *period)
            .await;
        let chat_categories = storage
            .clone()
            .as_category_storage()
            .get_chat_categories(chat_id)
            .await
            .unwrap_or_default();

        // Filter expenses for the category
        let filtered_expenses =
            filter_category_expenses(&category.to_string(), &chat_expenses, &chat_categories);

        // Calculate pagination
        let total_expenses = filtered_expenses.len();
        let total_pages = total_expenses.div_ceil(RECORDS_PER_PAGE);
        let max_page = total_pages.saturating_sub(1);
        let page_number = page.min(&max_page);

        // Calculate total amount for the category
        let total_amount: f64 = filtered_expenses.iter().map(|e| e.amount).sum();

        // Format category report with pagination (just the data)
        let report_text =
            format_single_category_report(&filtered_expenses, *page_number, RECORDS_PER_PAGE);

        // Build header with category name, period, page info, and total
        let message = if filtered_expenses.is_empty() {
            yoroolbot::markdown_format!(
                "*{}* \\(period: *{}*\\): No expenses in this category\\.",
                category.as_str(),
                &period.to_string()
            )
        } else if total_pages > 1 {
            yoroolbot::markdown_format!(
                "*{}* \\(period: *{}*\\), total `{}`,  page {}/{}\n{}",
                category.as_str(),
                &period.to_string(),
                total_amount,
                page_number + 1,
                total_pages,
                @code report_text
            )
        } else {
            yoroolbot::markdown_format!(
                "*{}* \\(period: *{}*\\), total `{}`\n{}",
                category.as_str(),
                &period.to_string(),
                total_amount,
                @code report_text
            )
        };

        // Create navigation buttons
        let mut nav_buttons = Vec::new();

        // Previous/Next buttons row
        let mut page_nav_row = Vec::new();
        if *page_number > 0 {
            // Active previous button
            page_nav_row.push(yoroolbot::storage::ButtonData::Callback(
                "◀️ Prev".to_string(),
                CommandReport {
                    period: Some(*period),
                    category: Some(category.clone()),
                    page: Some(page_number - 1),
                }
                .to_command_string(false),
            ));
        } else {
            // Inactive previous button
            page_nav_row.push(yoroolbot::storage::ButtonData::Callback(
                "◁ Prev".to_string(),
                "noop".to_string(),
            ));
        }

        if page_number + 1 < total_pages {
            // Active next button
            page_nav_row.push(yoroolbot::storage::ButtonData::Callback(
                "Next ▶️".to_string(),
                CommandReport {
                    period: Some(*period),
                    category: Some(category.clone()),
                    page: Some(page_number + 1),
                }
                .to_command_string(false),
            ));
        } else {
            // Inactive next button
            page_nav_row.push(yoroolbot::storage::ButtonData::Callback(
                "Next ▷".to_string(),
                "noop".to_string(),
            ));
        }

        nav_buttons.push(page_nav_row);

        // Back button row
        let back_button_row = vec![yoroolbot::storage::ButtonData::Callback(
            "↩️ Back to Summary".to_string(),
            CommandReport {
                period: Some(*period),
                category: None,
                page: None,
            }
            .to_command_string(false),
        )];
        nav_buttons.push(back_button_row);

        target
            .markdown_message_with_menu(message, nav_buttons)
            .await?;

        Ok(())
    }
}

impl From<CommandReport> for crate::commands::Command {
    fn from(cmd: CommandReport) -> Self {
        crate::commands::Command::Report(cmd)
    }
}
