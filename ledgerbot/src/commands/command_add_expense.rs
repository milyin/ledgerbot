use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::storages::{Expense, Storage, get_current_period};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddExpense {
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
}

impl CommandTrait for CommandAddExpense {
    type A = NaiveDate; // date (required)
    type B = String; // description (required, with escaped spaces)
    type C = Decimal; // amount (required)
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<Storage>;

    const NAME: &'static str = "add_expense";
    const PLACEHOLDERS: &[&'static str] = &["<date>", "<description>", "<amount>"];

    fn from_arguments(
        a: Option<Self::A>,
        b: Option<Self::B>,
        c: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddExpense {
            date: a,
            description: b,
            amount: c,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.date.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.description.as_ref()
    }

    fn param3(&self) -> Option<&Self::C> {
        self.amount.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _storage: Self::Context,
    ) -> ResponseResult<()> {
        // Generate usage string dynamically
        let usage = self.to_command_string(true);

        // Generate example commands dynamically
        let example1 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("Coffee".to_string()),
            amount: Some(Decimal::new(550, 2)), // 5.50
        }
        .to_command_string(false);

        let example2 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("My Lunch".to_string()),
            amount: Some(Decimal::new(1200, 2)), // 12.00
        }
        .to_command_string(false);

        let example3 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("Groceries".to_string()),
            amount: Some(Decimal::new(4530, 2)), // 45.30
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
        target: &CommandReplyTarget,
        _storage: Self::Context,
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
        target: &CommandReplyTarget,
        _storage: Self::Context,
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
        target: &CommandReplyTarget,
        storage: Self::Context,
        date: &NaiveDate,
        description: &String,
        amount: &Decimal,
    ) -> ResponseResult<()> {
        // Create expense record
        let expense = Expense::new(*date, description.clone(), *amount);

        // Get the current period for this chat
        let period = get_current_period(&storage, target.chat.id).await;

        // Store the expense in the selected period
        storage
            .clone()
            .expense_storage()
            .add_expenses(target.chat.id, period, vec![expense])
            .await;

        if !target.batch {
            // Send confirmation message
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

impl From<CommandAddExpense> for crate::commands::Command {
    fn from(cmd: CommandAddExpense) -> Self {
        crate::commands::Command::AddExpense(cmd)
    }
}
