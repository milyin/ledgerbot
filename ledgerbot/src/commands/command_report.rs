use std::sync::Arc;

use rust_decimal::Decimal;
use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format, markdown_string,
    storage::{self, ButtonData},
};

use crate::{
    commands::{
        follow_helper::validate_and_get_follow_access,
        report::{
            check_category_conflicts, filter_category_expenses, format_category_comparison,
            format_single_category_report,
        },
    },
    storages::{self, Category, ExpensePeriod, Stores},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandReport {
    pub period: Option<ExpensePeriod>,
    pub reference_period: Option<ExpensePeriod>,
    pub category: Option<Category>,
    pub page: Option<usize>,
}

impl CommandTrait for CommandReport {
    type A = ExpensePeriod;
    type B = ExpensePeriod;
    type C = Category;
    type D = usize;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<Stores>;

    const NAME: &'static str = "report";
    const PLACEHOLDERS: &[&'static str] = &["period", "reference_period", "category", "page"];

    fn from_arguments(
        period: Option<Self::A>,
        reference_period: Option<Self::B>,
        category: Option<Self::C>,
        page: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandReport {
            period,
            reference_period,
            category,
            page,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.period.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.reference_period.as_ref()
    }

    fn param3(&self) -> Option<&Self::C> {
        self.category.as_ref()
    }

    fn param4(&self) -> Option<&Self::D> {
        self.page.as_ref()
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
        let var_storage = storage.clone().variable_storage();
        let current_period: Option<ExpensePeriod> = var_storage.get(chat_id).await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        let mut prompt = follow_access
            .header_note
            .unwrap_or_else(|| markdown_string!(""));
        prompt = prompt
            + markdown_format!(
                "📊 *Report for period*\n\n\
                 Current period: *{}*\n\n\
                 Select a period to view its report:",
                current_period_str
            );

        // Show menu with available periods
        let storages_ = storage.storage(chat_id);
        let expense_storage = storages_.expenses();
        crate::menus::select_period::select_period(
            target,
            &expense_storage,
            prompt,
            |period| CommandReport {
                period: Some(*period),
                reference_period: None,
                category: None,
                page: None,
            },
            None::<CommandReport>,
            None::<yoroolbot::command_trait::NoopCommand>,
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
        // Forward to run2 with reference period same as selected period
        self.run2(target, storage, period, period).await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
        reference_period: &ExpensePeriod,
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

        // Get expenses for both periods
        let storage_ = storage.storage(chat_id);
        let expense_storage = storage_.expenses();
        let current_expenses = expense_storage.get_expenses(*period).await;
        let reference_expenses = expense_storage
            .get_expenses(*reference_period)
            .await;

        // Determine if we should show comparison (only if periods differ)
        let (reference_expenses_opt, reference_period_opt) = if period == reference_period {
            (None, None)
        } else {
            (Some(&reference_expenses[..]), Some(reference_period))
        };

        let chat_categories = storage
            .clone()
            .category_storage()
            .get_chat_categories(chat_id)
            .await
            .unwrap_or_default();

        let storage_ = storage.storage(chat_id);
        let all_expenses = storage_
            .expenses()
            .get_all_expenses()
            .await
            .into_iter()
            .map(|(_, expense)| expense)
            .collect::<Vec<_>>();

        if let Some(conflict_message) = check_category_conflicts(&all_expenses, &chat_categories) {
            target.markdown_message(conflict_message).await?;
            return Ok(());
        }

        // Build summary message (will show comparison if periods differ)
        let (category_summary, _) = format_category_comparison(
            reference_expenses_opt,
            &current_expenses,
            &chat_categories,
            reference_period_opt,
            period,
        );

        // Prepend follow header if present
        let mut summary_message = follow_access
            .header_note
            .unwrap_or_else(|| markdown_string!(""));
        summary_message = summary_message
            + category_summary
            + markdown_string!("\n_Select a period to comparison or view report by categories_");

        // Get available periods for buttons
        let periods = expense_storage.list_periods().await;

        // Create period buttons (4 per row)
        // When clicked, selected period becomes reference
        let period_buttons: Vec<yoroolbot::storage::ButtonData> = periods
            .iter()
            .map(|p| {
                yoroolbot::storage::ButtonData::Callback(
                    format!("📅 {}", p),
                    CommandReport {
                        period: Some(*period),
                        reference_period: Some(*p), // Selected becomes reference
                        category: None,
                        page: None,
                    }
                    .to_command_string(false),
                )
            })
            .collect();

        let mut buttons: Vec<Vec<yoroolbot::storage::ButtonData>> = period_buttons
            .chunks(4)
            .map(|chunk| chunk.to_vec())
            .collect();

        // Add action buttons row
        let mut action_row = Vec::new();

        // Add swap button only if periods differ
        if period != reference_period {
            action_row.push(ButtonData::Callback(
                "🔄 Swap".to_string(),
                CommandReport {
                    period: Some(*reference_period),
                    reference_period: Some(*period),
                    category: None,
                    page: None,
                }
                .to_command_string(false),
            ));
        }

        // Always add "Report by categories" button
        action_row.push(ButtonData::Callback(
            "📁 Report by categories".to_string(),
            CommandReport {
                period: Some(*period),
                reference_period: Some(*reference_period),
                category: Some(Category::None),
                page: None,
            }
            .to_command_string(false),
        ));

        buttons.push(action_row);

        // Send message with period selection menu
        target
            .markdown_message_with_menu(summary_message, buttons)
            .await?;

        Ok(())
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
        reference_period: &ExpensePeriod,
        category: &Category,
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
        let storage_ = storage.storage(chat_id);

        // If category is None, show summary with category buttons
        if category.is_none() {
            let chat_expenses = storage_
                .expenses()
                .get_expenses(*period)
                .await;
            let reference_expenses = storage_
                .expenses()
                .get_expenses(*reference_period)
                .await;

            // Determine if we should show comparison (only if periods differ)
            let (reference_expenses_opt, reference_period_opt) = if period == reference_period {
                (None, None)
            } else {
                (Some(&reference_expenses[..]), Some(reference_period))
            };

            let chat_categories = storage
                .clone()
                .category_storage()
                .get_chat_categories(chat_id)
                .await
                .unwrap_or_default();

            let all_expenses = storage_
                .expenses()
                .get_all_expenses()
                .await
                .into_iter()
                .map(|(_, expense)| expense)
                .collect::<Vec<_>>();

            if let Some(conflict_message) =
                check_category_conflicts(&all_expenses, &chat_categories)
            {
                target.markdown_message(conflict_message).await?;
                return Ok(());
            }

            // Show comparison summary and get list of categories
            let (category_summary, found_categories) = format_category_comparison(
                reference_expenses_opt,
                &chat_expenses,
                &chat_categories,
                reference_period_opt,
                period,
            );

            // Prepend follow header if present
            let mut message = follow_access
                .header_note
                .unwrap_or_else(|| markdown_string!(""));
            message = message + category_summary;

            // Create category buttons (4 per row)
            let mut buttons: Vec<Vec<yoroolbot::storage::ButtonData>> = Vec::new();
            let mut current_row: Vec<yoroolbot::storage::ButtonData> = Vec::new();

            for category in &found_categories {
                current_row.push(yoroolbot::storage::ButtonData::Callback(
                    format!("📁 {}", category.as_str()),
                    CommandReport {
                        period: Some(*period),
                        reference_period: Some(*reference_period),
                        category: Some(category.clone()),
                        page: None,
                    }
                    .to_command_string(false),
                ));

                // Start a new row after 4 buttons
                if current_row.len() == 4 {
                    buttons.push(current_row.clone());
                    current_row.clear();
                }
            }

            // Add remaining buttons if any
            if !current_row.is_empty() {
                buttons.push(current_row);
            }

            // Add navigation buttons row
            let mut nav_row = Vec::new();

            // Add swap button only if periods differ
            if period != reference_period {
                nav_row.push(ButtonData::Callback(
                    "🔄 Swap".to_string(),
                    CommandReport {
                        period: Some(*reference_period),
                        reference_period: Some(*period),
                        category: Some(Category::None),
                        page: None,
                    }
                    .to_command_string(false),
                ));
            }

            // Always add back button
            nav_row.push(ButtonData::Callback(
                "↩️ Back to Summary".to_string(),
                CommandReport {
                    period: Some(*period),
                    reference_period: Some(*reference_period),
                    category: None,
                    page: None,
                }
                .to_command_string(false),
            ));

            buttons.push(nav_row);

            // Send message with category menu
            target.markdown_message_with_menu(message, buttons).await?;

            return Ok(());
        }

        // Otherwise, show detailed category report (default to page 0)
        self.run4(target, storage, period, reference_period, category, &0)
            .await
    }

    async fn run4(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        period: &ExpensePeriod,
        reference_period: &ExpensePeriod,
        category: &Category,
        page: &usize,
    ) -> ResponseResult<()> {
        const RECORDS_PER_PAGE: usize = 25;

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

        let storage_ = storage.storage(chat_id);
        let chat_expenses = storage_
            .expenses()
            .get_expenses(*period)
            .await;
        let chat_categories = storage
            .clone()
            .category_storage()
            .get_chat_categories(chat_id)
            .await
            .unwrap_or_default();

        // Filter expenses for the category
        let filtered_expenses =
            filter_category_expenses(category, &chat_expenses, &chat_categories);

        // Calculate pagination
        let total_expenses = filtered_expenses.len();
        let total_pages = total_expenses.div_ceil(RECORDS_PER_PAGE);
        let max_page = total_pages.saturating_sub(1);
        let page_number = page.min(&max_page);

        // Calculate total amount for the category
        let total_amount: Decimal = filtered_expenses.iter().map(|e| e.amount).sum();

        // Format category report with pagination (just the data)
        let report_text =
            format_single_category_report(&filtered_expenses, *page_number, RECORDS_PER_PAGE);

        // Build header with category name, period, page info, and total
        let category_report = if filtered_expenses.is_empty() {
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
                total_amount.to_string(),
                page_number + 1,
                total_pages,
                @code report_text
            )
        } else {
            yoroolbot::markdown_format!(
                "*{}* \\(period: *{}*\\), total `{}`\n{}",
                category.as_str(),
                &period.to_string(),
                total_amount.to_string(),
                @code report_text
            )
        };

        // Prepend follow header if present
        let mut message = follow_access
            .header_note
            .unwrap_or_else(|| markdown_string!(""));
        message = message + category_report;

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
                    reference_period: Some(*reference_period),
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
                    reference_period: Some(*reference_period),
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

        // Back button row - goes back to category selection with both periods
        let mut back_button_row = Vec::new();

        // Add swap button only if periods differ
        if period != reference_period {
            message = message
                + markdown_format!(
                    "\n_Use *🔄 Swap* to switch to reference period {}_",
                    reference_period.to_string()
                );

            back_button_row.push(ButtonData::Callback(
                "🔄 Swap".to_string(),
                CommandReport {
                    period: Some(*reference_period),
                    reference_period: Some(*period),
                    category: Some(category.clone()),
                    page: Some(0),
                }
                .to_command_string(false),
            ));
        }

        // Always add back button
        back_button_row.push(ButtonData::Callback(
            "↩️ Back to Categories".to_string(),
            CommandReport {
                period: Some(*period),
                reference_period: Some(*reference_period),
                category: Some(Category::None),
                page: None,
            }
            .to_command_string(false),
        ));

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
