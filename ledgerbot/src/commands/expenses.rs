use chrono::{DateTime, TimeZone, Utc};
use yoroolbot::{markdown::MarkdownString, markdown_format};

use crate::storages::storage_traits::Expense;

/// Format timestamp as YYYY-MM-DD string
fn format_timestamp(timestamp: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}

/// Format expenses as a chronological list without category grouping
/// Returns Ok(Vec<MarkdownString>) with one or more messages (split if needed to avoid overflow),
/// or Err(MarkdownString) with error message
pub fn format_expenses_chronological(
    expenses: &[Expense],
) -> Result<Vec<MarkdownString>, MarkdownString> {
    if expenses.is_empty() {
        return Err(markdown_format!(
            "📝 No expenses recorded yet\\. Send a message like `2024\\-10\\-09 Coffee 5\\.50` to add one\\."
        ));
    }

    // Sort by timestamp (chronological order)
    let mut sorted_expenses = expenses.to_vec();
    sorted_expenses.sort_by_key(|e| e.timestamp);

    let mut messages = Vec::new();
    let mut current_message = MarkdownString::new();

    for expense in sorted_expenses {
        let date_str = format_timestamp(expense.timestamp);
        let expense_line = markdown_format!(
            "{} {} {}\n",
            &date_str,
            &expense.description,
            &expense.amount.to_string()
        );

        // Try to add the expense line to current message
        let mut test_message = current_message.clone();
        test_message.push(&expense_line);

        if test_message.is_truncated() {
            // Current message would overflow, start a new one
            if current_message.as_str().is_empty() {
                // Edge case: single expense line is too long, add it anyway
                current_message.push(&expense_line);
            }
            messages.push(current_message);
            current_message = MarkdownString::new();
            current_message.push(&expense_line);
        } else {
            // Line fits, update current message
            current_message = test_message;
        }
    }

    // Add the last message if it has content
    if !current_message.as_str().is_empty() {
        messages.push(current_message);
    }

    Ok(messages)
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
        // Function returns Ok with Vec<MarkdownString>
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert!(!messages.is_empty());

        // For small list, should be in a single message
        assert_eq!(messages.len(), 1);
        let content = messages[0].as_str();
        assert!(content.contains("Coffee"));
        assert!(content.contains("Lunch"));
        assert!(content.contains("Dinner"));
        assert!(content.contains("5\\.5"));
        assert!(content.contains("12"));
        assert!(content.contains("25"));
    }

    #[test]
    fn test_format_expenses_chronological_empty() {
        // Test with no expenses
        let expenses = Vec::new();
        let result = format_expenses_chronological(&expenses);

        // Should return Err with error message
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.as_str().contains("No expenses recorded yet"));
    }

    #[test]
    fn test_format_expenses_chronological_large_list() {
        // Create a large list of expenses that should trigger message splitting
        // Each expense line is approximately 40-50 characters
        // Telegram limit is 4096 characters, so we need ~100+ expenses
        let base_timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let mut expenses = Vec::new();

        for i in 0..150 {
            expenses.push(Expense {
                description: format!("Expense number {}", i),
                amount: 10.50 + (i as f64),
                timestamp: base_timestamp + (i * 86400), // One day apart
            });
        }

        let result = format_expenses_chronological(&expenses);

        // Should return Ok with multiple messages
        assert!(result.is_ok());
        let messages = result.unwrap();

        // Should have split into multiple messages
        assert!(
            messages.len() > 1,
            "Expected multiple messages, got {}",
            messages.len()
        );

        // All messages should be non-empty
        for (idx, message) in messages.iter().enumerate() {
            assert!(!message.as_str().is_empty(), "Message {} is empty", idx);
        }

        // Verify all expenses are included across all messages
        let combined = messages
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join("");

        // Check a few sample expenses are present
        assert!(combined.contains("Expense number 0"));
        assert!(combined.contains("Expense number 50"));
        assert!(combined.contains("Expense number 149"));
    }
}
