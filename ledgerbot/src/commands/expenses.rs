use chrono::{DateTime, TimeZone, Utc};
use yoroolbot::{markdown::MarkdownString, markdown_format};

use crate::storage_traits::Expense;

/// Format timestamp as YYYY-MM-DD string
fn format_timestamp(timestamp: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}

/// Format expenses as a chronological list without category grouping
/// Returns Ok(String) with plain text list of expenses, or Err(MarkdownString) with error message
pub fn format_expenses_chronological(expenses: &[Expense]) -> Result<MarkdownString, MarkdownString> {
    if expenses.is_empty() {
        return Err(markdown_format!(
            "📝 No expenses recorded yet\\. Send a message like `2024\\-10\\-09 Coffee 5\\.50` to add one\\."
        ));
    }

    // Sort by timestamp (chronological order)
    let mut sorted_expenses = expenses.to_vec();
    sorted_expenses.sort_by_key(|e| e.timestamp);

    let mut result = MarkdownString::new();

    for expense in sorted_expenses {
        let date_str = format_timestamp(expense.timestamp);
        result = result + &markdown_format!(
            "{} {} {}\n",
            &date_str,
            &expense.description,
            &expense.amount.to_string()
        );
    }

    Ok(result)
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
}
