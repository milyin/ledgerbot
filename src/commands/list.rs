use teloxide::{
    prelude::*,
    types::Message,
};
use chrono::{DateTime, TimeZone, Utc};

use crate::storage::{Expense, ExpenseStorage, get_chat_expenses};

/// Format timestamp as YYYY-MM-DD string
fn format_timestamp(timestamp: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}

/// List all expenses chronologically without category grouping
pub async fn list_command(bot: Bot, msg: Message, storage: ExpenseStorage) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = get_chat_expenses(&storage, chat_id).await;
    let expenses_list = format_expenses_chronological(&chat_expenses);

    bot.send_message(chat_id, expenses_list)
        .await?;
    Ok(())
}

/// Format expenses as a chronological list without category grouping
/// Output format: "date description price"
pub fn format_expenses_chronological(expenses: &[Expense]) -> String {
    if expenses.is_empty() {
        return "No expenses recorded yet.".to_string();
    }

    // Sort by timestamp (chronological order)
    let mut sorted_expenses = expenses.to_vec();
    sorted_expenses.sort_by_key(|e| e.timestamp);

    let mut result = String::new();

    for expense in sorted_expenses {
        let date_str = format_timestamp(expense.timestamp);
        result.push_str(&format!(
            "{} {} {}\n",
            &date_str,
            &expense.description,
            &expense.amount.to_string()
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::{commands::list::format_expenses_chronological, storage::Expense};

    #[test]
    fn test_format_expenses_chronological() {
        // Create test expenses with different timestamps
        let timestamp1 = 1609459200; // 2021-01-01 00:00:00 UTC
        let timestamp2 = 1609545600; // 2021-01-02 00:00:00 UTC
        let timestamp3 = 1609632000; // 2021-01-03 00:00:00 UTC

        let expenses = vec![
            Expense {
                description: "Lunch".to_string(),
                amount: 12.00,
                timestamp: timestamp2,
            },
            Expense {
                description: "Coffee".to_string(),
                amount: 5.50,
                timestamp: timestamp1,
            },
            Expense {
                description: "Dinner".to_string(),
                amount: 25.00,
                timestamp: timestamp3,
            },
        ];

        let result = format_expenses_chronological(&expenses);

        // Check that expenses are listed in chronological order
        // Function returns plain format: "date description amount"
        assert_eq!(
            result,
            "2021-01-01 Coffee 5.5\n2021-01-02 Lunch 12\n2021-01-03 Dinner 25\n"
        );
    }

    #[test]
    fn test_format_expenses_chronological_empty() {
        // Test with no expenses
        let expenses = Vec::new();
        let result = format_expenses_chronological(&expenses);
        assert_eq!(result, "No expenses recorded yet.");
    }
}