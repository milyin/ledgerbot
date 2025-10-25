use std::sync::Arc;

use chrono::NaiveDate;
use teloxide::prelude::ResponseResult;
use yoroolbot::markdown_format;

use crate::commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg};
use crate::storage_traits::ExpenseStorageTrait;

/// Escape spaces in a string for command parameters
pub fn escape_spaces(s: &str) -> String {
    s.replace(' ', "\\ ")
}

/// Unescape spaces in a string from command parameters
pub fn unescape_spaces(s: &str) -> String {
    s.replace("\\ ", " ")
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddExpense {
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub amount: Option<f64>,
}

impl CommandTrait for CommandAddExpense {
    type A = NaiveDate; // date (required)
    type B = String;    // description (required, with escaped spaces)
    type C = f64;       // amount (required)
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn ExpenseStorageTrait>;

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
            description: b.map(|s| unescape_spaces(&s)),
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
            amount: Some(5.50),
        }
        .to_command_string(false);

        let example2 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("My Lunch".to_string()),
            amount: Some(12.00),
        }
        .to_command_string(false);

        let example3 = CommandAddExpense {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            description: Some("Groceries".to_string()),
            amount: Some(45.30),
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
                usage, example1, example2, example3
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
            .send_markdown_message(markdown_format!(
                "❌ Missing amount\\. Usage: `{}`",
                usage
            ))
            .await?;
        Ok(())
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        date: &NaiveDate,
        description: &String,
        amount: &f64,
    ) -> ResponseResult<()> {
        // Use provided date
        let timestamp = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        // Store the expense
        storage
            .add_expense(target.chat.id, description, *amount, timestamp)
            .await;

        // Send confirmation message
        target
            .send_markdown_message(markdown_format!(
                "✅ Expense added: {} {} {}",
                date.to_string(),
                description,
                amount.to_string()
            ))
            .await?;

        Ok(())
    }

    fn to_command_string(&self, with_placeholders: bool) -> String {
        let cmd = format!("/{}", Self::NAME);

        let date_str = if with_placeholders && self.date.is_none() {
            Self::PLACEHOLDERS[0].to_string()
        } else if let Some(ref date) = self.date {
            date.format("%Y-%m-%d").to_string()
        } else {
            Self::PLACEHOLDERS[0].to_string()
        };

        let description_str = if with_placeholders && self.description.is_none() {
            Self::PLACEHOLDERS[1].to_string()
        } else if let Some(ref desc) = self.description {
            escape_spaces(desc)
        } else {
            Self::PLACEHOLDERS[1].to_string()
        };

        let amount_str = if with_placeholders && self.amount.is_none() {
            Self::PLACEHOLDERS[2].to_string()
        } else if let Some(amt) = self.amount {
            amt.to_string()
        } else {
            Self::PLACEHOLDERS[2].to_string()
        };

        format!("{} {} {} {}", cmd, date_str, description_str, amount_str)
    }
}

impl From<CommandAddExpense> for crate::commands::Command {
    fn from(cmd: CommandAddExpense) -> Self {
        crate::commands::Command::AddExpense(cmd)
    }
}
