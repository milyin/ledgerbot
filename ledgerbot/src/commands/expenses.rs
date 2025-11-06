use yoroolbot::{markdown::MarkdownString, markdown_format};

use crate::storages::Expense;

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

    // Sort by date (chronological order)
    let mut sorted_expenses = expenses.to_vec();
    sorted_expenses.sort_by_key(|e| e.date);

    let mut messages = Vec::new();
    let mut current_message = MarkdownString::new();

    for expense in sorted_expenses {
        let date_str = expense.date.format("%Y-%m-%d").to_string();
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
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use crate::{commands::expenses::format_expenses_chronological, storages::Expense};

    #[test]
    fn test_format_expenses_chronological() {
        // Create test expenses with different dates
        let date1 = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2021, 1, 2).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2021, 1, 3).unwrap();

        let expenses = vec![
            Expense::new(date2, "Lunch".to_string(), Decimal::new(1200, 2)),
            Expense::new(date1, "Coffee".to_string(), Decimal::new(550, 2)),
            Expense::new(date3, "Dinner".to_string(), Decimal::new(2500, 2)),
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
        let base_date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
        let mut expenses = Vec::new();

        for i in 0..150 {
            let date = base_date + chrono::Days::new(i);
            expenses.push(Expense::new(
                date,
                format!("Expense number {}", i),
                Decimal::new(1050 + i as i64 * 100, 2),
            ));
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
