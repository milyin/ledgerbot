use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::prelude::ResponseResult;

use crate::{
    impl_command_execute_3, impl_command_io,
    storages::{Expense, Storage, get_current_period},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandAddExpense {
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
}

impl CommandAddExpense {
    async fn run0(&self, target: &CommandContext, _storage: Arc<Storage>) -> ResponseResult<()> {
        let usage = self.to_command_string(true);

        let example1 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("Coffee".to_string()),
            amount: Some(Decimal::new(550, 2)),
        }
        .to_command_string(false);

        let example2 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("My Lunch".to_string()),
            amount: Some(Decimal::new(1200, 2)),
        }
        .to_command_string(false);

        let example3 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("Groceries".to_string()),
            amount: Some(Decimal::new(4530, 2)),
        }
        .to_command_string(false);

        target
            .send_markdown_message(markdown_format!(
                "📝 Usage: `{}`\n\n\
                 Examples:\n\
                 • `{}`\n\
                 • `{}` \\(with escaped space\\)\n\
                 • `{}`\n\n\
                 Note: Use backslash to escape spaces in description: `My\\\\ Lunch`",
                usage,
                example1,
                example2,
                example3
            ))
            .await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        _storage: Arc<Storage>,
        _date: &NaiveDate,
    ) -> ResponseResult<()> {
        let usage = self.to_command_string(true);
        target
            .send_markdown_message(markdown_format!(
                "❌ Missing description and amount\\. Usage: `{}`",
                usage
            ))
            .await?;
        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandContext,
        _storage: Arc<Storage>,
        _date: &NaiveDate,
        _description: &String,
    ) -> ResponseResult<()> {
        let usage = self.to_command_string(true);
        target
            .send_markdown_message(markdown_format!("❌ Missing amount\\. Usage: `{}`", usage))
            .await?;
        Ok(())
    }

    async fn run3(
        &self,
        target: &CommandContext,
        storage: Arc<Storage>,
        date: &NaiveDate,
        description: &String,
        amount: &Decimal,
    ) -> ResponseResult<()> {
        let expense = Expense::new(*date, description.clone(), *amount);
        let period = get_current_period(&storage).await;
        storage.expenses().add_expenses(period, vec![expense]).await;

        if !target.batch {
            target
                .send_markdown_message(markdown_format!(
                    "✅ Expense added: {} {} {}",
                    date.to_string(),
                    description,
                    amount.to_string()
                ))
                .await?;
        }

        Ok(())
    }
}

impl_command_io!(
    CommandAddExpense,
    "add_expense",
    ["<date>", "<description>", "<amount>"],
    date: NaiveDate,
    description: String,
    amount: Decimal
);
impl_command_execute_3!(CommandAddExpense, Arc<Storage>, date, description, amount);

impl From<CommandAddExpense> for crate::commands::Command {
    fn from(cmd: CommandAddExpense) -> Self {
        crate::commands::Command::AddExpense(cmd)
    }
}
