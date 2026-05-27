use std::sync::Arc;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    commands::report::{
        check_category_conflicts, filter_category_expenses, format_category_comparison,
        format_single_category_report,
    },
    impl_command_execute_4, impl_command_io,
    menus::common::make_follow_status_message,
    storages::{Category, ExpensePeriod, StorageReadonly},
    ui::{ButtonData, CommandContext},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandReport {
    pub period: Option<ExpensePeriod>,
    pub reference_period: Option<ExpensePeriod>,
    pub category: Option<Category>,
    pub page: Option<usize>,
}

impl CommandReport {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
    ) -> ResponseResult<()> {
        let var_storage = storage.variables();
        let current_period: Option<ExpensePeriod> = var_storage.get().await;

        let current_period_str = if let Some(period) = current_period {
            period.to_string()
        } else {
            ExpensePeriod::current().to_string()
        };

        let follow_header = make_follow_status_message(&storage);
        let prompt = follow_header
            + markdown_format!(
                "📊 *Report for period*\n\n\
                 Current period: *{}*\n\n\
                 Select a period to view its report:",
                current_period_str
            );

        let expense_storage = storage.expenses_readonly();
        crate::menus::select_period::select_period(
            target,
            &expense_storage,
            prompt,
            |period| {
                CommandReport {
                    period: Some(*period),
                    reference_period: None,
                    category: None,
                    page: None,
                }
                .into()
            },
            None,
            None,
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
        period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        self.run2(target, storage, period, period).await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
        period: &ExpensePeriod,
        reference_period: &ExpensePeriod,
    ) -> ResponseResult<()> {
        let expense_storage = storage.expenses_readonly();
        let current_expenses = expense_storage.get_expenses(*period).await;
        let reference_expenses = expense_storage.get_expenses(*reference_period).await;

        let (reference_expenses_opt, reference_period_opt) = if period == reference_period {
            (None, None)
        } else {
            (Some(&reference_expenses[..]), Some(reference_period))
        };

        let chat_categories = storage
            .categories_readonly()
            .get_categories()
            .await
            .unwrap_or_default();

        let all_expenses = storage
            .expenses_readonly()
            .get_all_expenses()
            .await
            .into_iter()
            .map(|(_, expense)| expense)
            .collect::<Vec<_>>();

        if let Some(conflict_message) = check_category_conflicts(&all_expenses, &chat_categories) {
            target.markdown_message(conflict_message).await?;
            return Ok(());
        }

        let (category_summary, _) = format_category_comparison(
            reference_expenses_opt,
            &current_expenses,
            &chat_categories,
            reference_period_opt,
            period,
        );

        let follow_header = make_follow_status_message(&storage);
        let summary_message = follow_header
            + category_summary
            + markdown_string!("\n_Select a period to comparison or view report by categories_");

        let periods = expense_storage.list_periods().await;
        let period_buttons: Vec<ButtonData> = periods
            .iter()
            .map(|p| {
                ButtonData::Command(
                    format!("📅 {}", p),
                    CommandReport {
                        period: Some(*period),
                        reference_period: Some(*p),
                        category: None,
                        page: None,
                    }
                    .into(),
                )
            })
            .collect();

        let mut buttons: Vec<Vec<ButtonData>> = period_buttons
            .chunks(4)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut action_row = Vec::new();

        if period != reference_period {
            action_row.push(ButtonData::Command(
                "🔄 Swap".to_string(),
                CommandReport {
                    period: Some(*reference_period),
                    reference_period: Some(*period),
                    category: None,
                    page: None,
                }
                .into(),
            ));
        }

        action_row.push(ButtonData::Command(
            "📁 Report by categories".to_string(),
            CommandReport {
                period: Some(*period),
                reference_period: Some(*reference_period),
                category: Some(Category::None),
                page: None,
            }
            .into(),
        ));

        buttons.push(action_row);
        target
            .markdown_message_with_menu(summary_message, buttons)
            .await?;
        Ok(())
    }

    async fn run3(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
        period: &ExpensePeriod,
        reference_period: &ExpensePeriod,
        category: &Category,
    ) -> ResponseResult<()> {
        if category.is_none() {
            let chat_expenses = storage.expenses_readonly().get_expenses(*period).await;
            let reference_expenses = storage
                .expenses_readonly()
                .get_expenses(*reference_period)
                .await;

            let (reference_expenses_opt, reference_period_opt) = if period == reference_period {
                (None, None)
            } else {
                (Some(&reference_expenses[..]), Some(reference_period))
            };

            let chat_categories = storage
                .categories_readonly()
                .get_categories()
                .await
                .unwrap_or_default();

            let all_expenses = storage
                .expenses_readonly()
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

            let (category_summary, found_categories) = format_category_comparison(
                reference_expenses_opt,
                &chat_expenses,
                &chat_categories,
                reference_period_opt,
                period,
            );

            let follow_header = make_follow_status_message(&storage);
            let message = follow_header + category_summary;

            let mut buttons: Vec<Vec<ButtonData>> = Vec::new();
            let mut current_row: Vec<ButtonData> = Vec::new();

            for category in &found_categories {
                current_row.push(ButtonData::Command(
                    format!("📁 {}", category.as_str()),
                    CommandReport {
                        period: Some(*period),
                        reference_period: Some(*reference_period),
                        category: Some(category.clone()),
                        page: None,
                    }
                    .into(),
                ));

                if current_row.len() == 4 {
                    buttons.push(current_row.clone());
                    current_row.clear();
                }
            }

            if !current_row.is_empty() {
                buttons.push(current_row);
            }

            let mut nav_row = Vec::new();

            if period != reference_period {
                nav_row.push(ButtonData::Command(
                    "🔄 Swap".to_string(),
                    CommandReport {
                        period: Some(*reference_period),
                        reference_period: Some(*period),
                        category: Some(Category::None),
                        page: None,
                    }
                    .into(),
                ));
            }

            nav_row.push(ButtonData::Command(
                "↩️ Back to Summary".to_string(),
                CommandReport {
                    period: Some(*period),
                    reference_period: Some(*reference_period),
                    category: None,
                    page: None,
                }
                .into(),
            ));

            buttons.push(nav_row);
            target.markdown_message_with_menu(message, buttons).await?;
            return Ok(());
        }

        self.run4(target, storage, period, reference_period, category, &0)
            .await
    }

    async fn run4(
        &self,
        target: &CommandContext,
        storage: Arc<StorageReadonly>,
        period: &ExpensePeriod,
        reference_period: &ExpensePeriod,
        category: &Category,
        page: &usize,
    ) -> ResponseResult<()> {
        const RECORDS_PER_PAGE: usize = 25;

        let chat_expenses = storage.expenses_readonly().get_expenses(*period).await;
        let chat_categories = storage
            .categories_readonly()
            .get_categories()
            .await
            .unwrap_or_default();

        let filtered_expenses =
            filter_category_expenses(category, &chat_expenses, &chat_categories);
        let total_expenses = filtered_expenses.len();
        let total_pages = total_expenses.div_ceil(RECORDS_PER_PAGE);
        let max_page = total_pages.saturating_sub(1);
        let page_number = page.min(&max_page);
        let total_amount: Decimal = filtered_expenses.iter().map(|e| e.amount).sum();
        let report_text =
            format_single_category_report(&filtered_expenses, *page_number, RECORDS_PER_PAGE);

        let category_report = if filtered_expenses.is_empty() {
            markdown_format!(
                "*{}* \\(period: *{}*\\): No expenses in this category\\.",
                category.as_str(),
                &period.to_string()
            )
        } else if total_pages > 1 {
            markdown_format!(
                "*{}* \\(period: *{}*\\), total `{}`,  page {}/{}\n{}",
                category.as_str(),
                &period.to_string(),
                total_amount.to_string(),
                page_number + 1,
                total_pages,
                @code report_text
            )
        } else {
            markdown_format!(
                "*{}* \\(period: *{}*\\), total `{}`\n{}",
                category.as_str(),
                &period.to_string(),
                total_amount.to_string(),
                @code report_text
            )
        };

        let follow_header = make_follow_status_message(&storage);
        let mut message = follow_header + category_report;
        let mut nav_buttons = Vec::new();

        let mut page_nav_row = Vec::new();
        if *page_number > 0 {
            page_nav_row.push(ButtonData::Command(
                "◀️ Prev".to_string(),
                CommandReport {
                    period: Some(*period),
                    reference_period: Some(*reference_period),
                    category: Some(category.clone()),
                    page: Some(page_number - 1),
                }
                .into(),
            ));
        } else {
            page_nav_row.push(ButtonData::RawCallback(
                "◁ Prev".to_string(),
                "noop".to_string(),
            ));
        }

        if page_number + 1 < total_pages {
            page_nav_row.push(ButtonData::Command(
                "Next ▶️".to_string(),
                CommandReport {
                    period: Some(*period),
                    reference_period: Some(*reference_period),
                    category: Some(category.clone()),
                    page: Some(page_number + 1),
                }
                .into(),
            ));
        } else {
            page_nav_row.push(ButtonData::RawCallback(
                "Next ▷".to_string(),
                "noop".to_string(),
            ));
        }

        nav_buttons.push(page_nav_row);

        let mut back_button_row = Vec::new();
        if period != reference_period {
            message = message
                + markdown_format!(
                    "\n_Use *🔄 Swap* to switch to reference period {}_",
                    reference_period.to_string()
                );

            back_button_row.push(ButtonData::Command(
                "🔄 Swap".to_string(),
                CommandReport {
                    period: Some(*reference_period),
                    reference_period: Some(*period),
                    category: Some(category.clone()),
                    page: Some(0),
                }
                .into(),
            ));
        }

        back_button_row.push(ButtonData::Command(
            "↩️ Back to Categories".to_string(),
            CommandReport {
                period: Some(*period),
                reference_period: Some(*reference_period),
                category: Some(Category::None),
                page: None,
            }
            .into(),
        ));

        nav_buttons.push(back_button_row);
        target
            .markdown_message_with_menu(message, nav_buttons)
            .await?;

        Ok(())
    }
}

impl_command_io!(
    CommandReport,
    "report",
    ["period", "reference_period", "category", "page"],
    period: ExpensePeriod,
    reference_period: ExpensePeriod,
    category: Category,
    page: usize
);
impl_command_execute_4!(
    CommandReport,
    Arc<StorageReadonly>,
    period,
    reference_period,
    category,
    page
);

impl From<CommandReport> for crate::commands::Command {
    fn from(cmd: CommandReport) -> Self {
        crate::commands::Command::Report(cmd)
    }
}
