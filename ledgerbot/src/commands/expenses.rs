use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use teloxide::{prelude::*, types::Message, utils::command::ParseError};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format, markdown_string};

use crate::storage_traits::{Expense, ExpenseStorageTrait};

/// Format timestamp as YYYY-MM-DD string
fn format_timestamp(timestamp: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}

/// Custom parser for expense command (date, description, amount)
pub type ExpenseParams = (Option<NaiveDate>, Option<String>, Option<f64>);
pub fn parse_expense(s: String) -> Result<ExpenseParams, ParseError> {
    // Take only the first line to prevent multi-line capture
    let first_line = s.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return Ok((None, None, None));
    }

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok((None, None, None));
    }

    // The last part is always the amount
    let last_part = parts.last().unwrap();
    let amount = last_part.parse::<f64>().ok();

    if amount.is_none() {
        // If the last part is not a number, consider the whole string as description
        return Ok((None, Some(first_line.to_string()), None));
    }

    let mut description_parts = &parts[..parts.len() - 1];

    // The first part might be a date
    let date = if !description_parts.is_empty() {
        if let Ok(d) = NaiveDate::parse_from_str(description_parts[0], "%Y-%m-%d") {
            description_parts = &description_parts[1..];
            Some(d)
        } else {
            None
        }
    } else {
        None
    };

    if description_parts.is_empty() {
        return Ok((date, None, amount));
    }

    let description = description_parts.join(" ");

    Ok((date, Some(description), amount))
}

/// List all expenses chronologically without category grouping
pub async fn list_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn ExpenseStorageTrait>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = storage.get_chat_expenses(chat_id).await;
    let expenses_list = format_expenses_chronological(&chat_expenses);

    bot.markdown_message(chat_id,  None, markdown_format!("{}", expenses_list))
        .await?;
    Ok(())
}

/// Format expenses as a chronological list without category grouping
/// Output format: "date description price"
fn format_expenses_chronological(expenses: &[Expense]) -> String {
    if expenses.is_empty() {
        return "📝 No expenses recorded yet. Send a message like `2024-10-09 Coffee 5.50` to add one.".to_string();
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

/// Handle expense command with date, description, and amount
pub async fn expense_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn ExpenseStorageTrait>,
    date: Option<NaiveDate>,
    description: Option<String>,
    amount: Option<f64>,
    silent: bool,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Get message timestamp for default date
    let message_timestamp = msg.forward_date().unwrap_or(msg.date).timestamp();

    // Validate and parse parameters
    match (description, amount) {
        (Some(desc), Some(amount_val)) => {
            // Determine timestamp
            let timestamp = if let Some(ref date_val) = date {
                // Try to parse the date
                date_val.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()
            } else {
                message_timestamp
            };

            // Store the expense
            storage
                .add_expense(chat_id, &desc, amount_val, timestamp)
                .await;

            // Send confirmation message only if not silent
            if !silent {
                // Format date for display
                let date_display = if let Some(d) = date {
                    d.to_string()
                } else {
                    use chrono::{DateTime, Utc};
                    let dt: DateTime<Utc> =
                        DateTime::from_timestamp(timestamp, 0).unwrap_or_default();
                    dt.format("%Y-%m-%d").to_string()
                };

                bot.markdown_message(
                    chat_id,
                    None,
                    markdown_format!(
                        "✅ Expense added: {} {} {}",
                        date_display,
                        desc,
                        amount_val.to_string()
                    ),
                )
                .await?;
            }
        }
        (Some(desc), None) => {
            bot.markdown_message(
                chat_id,
                None,
                markdown_format!(
                    "❌ Invalid amount for `{}`\\. Please provide a valid number\\.",
                    desc
                ),
            )
            .await?;
        }
        _ => {
            // Handle other cases if necessary, e.g., no description
        }
    }

    Ok(())
}

/// Clear all expenses
pub async fn clear_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn ExpenseStorageTrait>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    storage.clear_chat_expenses(chat_id).await;

    bot.markdown_message(chat_id, None, markdown_string!("🗑️ All expenses cleared\\!"))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{commands::expenses::format_expenses_chronological, storage_traits::Expense};

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
        assert_eq!(
            result,
            "📝 No expenses recorded yet. Send a message like `2024-10-09 Coffee 5.50` to add one."
        );
    }
}
