use std::collections::HashMap;

use yoroolbot::{markdown::MarkdownString, markdown_format, markdown_string};

use crate::{storage_traits::Expense, utils::format_timestamp};

/// Represents a conflict where an expense matches multiple categories
#[derive(Debug, Clone)]
struct CategoryConflict {
    expense: Expense,
    matching_categories: Vec<(String, String)>, // (category_name, matched_pattern)
}

/// Check if any expense matches multiple categories
/// Returns Some with formatted error message if conflicts are found, None otherwise
pub fn check_category_conflicts(
    expenses: &[Expense],
    categories: &HashMap<String, Vec<String>>,
) -> Option<MarkdownString> {
    let mut conflicts: Vec<CategoryConflict> = Vec::new();

    // Build regex matchers for each category
    let category_matchers: Vec<(String, Vec<(String, regex::Regex)>)> = categories
        .iter()
        .map(|(name, patterns)| {
            let regexes: Vec<(String, regex::Regex)> = patterns
                .iter()
                .filter_map(|pattern| {
                    regex::Regex::new(pattern)
                        .ok()
                        .map(|re| (pattern.clone(), re))
                })
                .collect();
            (name.clone(), regexes)
        })
        .collect();

    // Check each expense for conflicts
    for expense in expenses {
        let mut matching_categories: Vec<(String, String)> = Vec::new();

        // Find all categories that match this expense
        for (category_name, regexes) in &category_matchers {
            for (pattern, re) in regexes {
                if re.is_match(&expense.description) {
                    matching_categories.push((category_name.clone(), pattern.clone()));
                    break; // Only add category once, even if multiple patterns match
                }
            }
        }

        // If expense matches more than one category, it's a conflict
        if matching_categories.len() > 1 {
            conflicts.push(CategoryConflict {
                expense: expense.clone(),
                matching_categories,
            });
        }
    }

    // If there are conflicts, format and return error message
    if !conflicts.is_empty() {
        let mut error_message = markdown_string!("❌ *Category Conflicts Detected*\n\n");
        error_message = error_message
            + markdown_string!(
                "The following expenses match multiple categories\\.\n\
                 Please adjust your filters to avoid overlapping categories\\.\n\n"
            );

        for conflict in conflicts {
            let date_str = format_timestamp(conflict.expense.timestamp);
            error_message = error_message
                + markdown_format!(
                    "📝 *Expense:* {} {} {}\n",
                    &*date_str,
                    &*conflict.expense.description,
                    conflict.expense.amount
                );
            error_message = error_message + markdown_string!("*Matching categories:*\n");
            for (category_name, pattern) in conflict.matching_categories {
                error_message = error_message
                    + markdown_format!("  • {} \\(filter: `{}`\\)\n", &*category_name, &*pattern);
            }
            error_message = error_message + markdown_string!("\n");
        }

        return Some(error_message);
    }

    None
}

/// Format expenses as multiple messages, one per category, plus a final total message
pub fn format_expenses_by_category(
    expenses: &[Expense],
    categories: &HashMap<String, Vec<String>>,
) -> Vec<MarkdownString> {
    if expenses.is_empty() {
        return vec![markdown_string!("No expenses recorded yet\\.")];
    }

    let mut messages = Vec::new();

    // Build regex matchers for each category (from all patterns)
    let category_matchers: Vec<(String, Vec<regex::Regex>)> = categories
        .iter()
        .map(|(name, patterns)| {
            let regexes: Vec<regex::Regex> = patterns
                .iter()
                .filter_map(|pattern| regex::Regex::new(pattern).ok())
                .collect();
            (name.clone(), regexes)
        })
        .collect();

    // Group expenses by category
    let mut categorized: HashMap<String, Vec<Expense>> = HashMap::new();
    let mut uncategorized: Vec<Expense> = Vec::new();

    for expense in expenses.iter() {
        let mut matched = false;

        // Try to match against each category
        for (category_name, regexes) in &category_matchers {
            // Check if description matches any of the patterns in this category
            if regexes.iter().any(|re| re.is_match(&expense.description)) {
                categorized
                    .entry(category_name.clone())
                    .or_default()
                    .push(expense.clone());
                matched = true;
                break; // Each expense goes into first matching category
            }
        }

        if !matched {
            uncategorized.push(expense.clone());
        }
    }

    // Sort category names for consistent output
    let mut category_names: Vec<String> = categorized.keys().cloned().collect();
    category_names.sort();

    let mut total = 0.0;
    let mut category_subtotals: Vec<(String, f64)> = Vec::new();

    // Create a message for each category
    for category_name in category_names {
        if let Some(items) = categorized.get(&category_name) {
            let (section, category_total) = format_category_message(&category_name, items);
            messages.push(section);
            category_subtotals.push((category_name.clone(), category_total));
            total += category_total;
        }
    }

    // Create a message for uncategorized expenses
    if !uncategorized.is_empty() {
        let (section, category_total) = format_category_message("Other", &uncategorized);
        messages.push(section);
        category_subtotals.push(("Other".to_string(), category_total));
        total += category_total;
    }

    // Add final total message with category breakdown as a table
    if !messages.is_empty() {
        // Find the maximum category name length for alignment
        let max_name_len = category_subtotals
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0)
            .max(5); // At least as wide as "Total"

        // Build the table content
        let mut table_lines = Vec::new();

        // Add each category row
        for (category_name, subtotal) in &category_subtotals {
            let padded_name = format!("{:<width$}", category_name, width = max_name_len);
            let amount_str = format!("{:>10.2}", subtotal);
            table_lines.push(format!("{} {}", padded_name, amount_str));
        }

        // Add separator line
        table_lines.push("-".repeat(max_name_len + 11));

        // Add total row
        let total_label = format!("{:<width$}", "Total", width = max_name_len);
        let total_amount = format!("{:>10.2}", total);
        table_lines.push(format!("{} {}", total_label, total_amount));

        // Join all lines and use @code modifier to wrap in code block
        let table_content = table_lines.join("\n");
        let total_message = markdown_format!("{}", @code table_content);

        messages.push(total_message);
    }

    messages
}

/// Helper function to format a single category as a standalone message
fn format_category_message(category_name: &str, expenses: &[Expense]) -> (MarkdownString, f64) {
    let mut category_total = 0.0;

    // Find the maximum description length for alignment
    let max_desc_len = expenses
        .iter()
        .map(|e| e.description.len())
        .max()
        .unwrap_or(0)
        .max(9); // At least as wide as "Subtotal:"

    // Build the table content
    let mut table_lines = Vec::new();

    for expense in expenses {
        let date_str = format_timestamp(expense.timestamp);
        let padded_desc = format!("{:<width$}", expense.description, width = max_desc_len);
        let amount_str = format!("{:>10.2}", expense.amount);
        table_lines.push(format!("{}  {} {}", date_str.as_str(), padded_desc, amount_str));
        category_total += expense.amount;
    }

    // Add separator line
    table_lines.push("-".repeat(date_str_len() + 2 + max_desc_len + 11));

    // Add subtotal row
    let subtotal_label = format!("{:<width$}", "Subtotal:", width = date_str_len() + 2 + max_desc_len);
    let subtotal_amount = format!("{:>10.2}", category_total);
    table_lines.push(format!("{} {}", subtotal_label, subtotal_amount));

    // Join all lines and use @code modifier to wrap in code block
    let table_content = table_lines.join("\n");
    let section = markdown_format!("*{}*:\n{}", category_name, @code table_content);

    (section, category_total)
}

/// Helper function to get the length of the formatted date string
fn date_str_len() -> usize {
    // Date format is "YYYY-MM-DD" which is always 10 characters
    10
}

/// Helper function to format a single category section with its expenses (for single-message report)
#[allow(dead_code)]
fn format_category_section(category_name: &str, expenses: &[Expense]) -> (MarkdownString, f64) {
    let (mut section, total) = format_category_message(category_name, expenses);
    section = section + markdown_string!("\n\n");
    (section, total)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn test_format_expenses_by_category_returns_multiple_messages() {
        // Test that the function returns multiple messages, one per category
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expenses = vec![
            Expense {
                description: "Coffee".to_string(),
                amount: 5.50,
                timestamp,
            },
            Expense {
                description: "Groceries".to_string(),
                amount: 25.00,
                timestamp,
            },
            Expense {
                description: "Tea".to_string(),
                amount: 3.00,
                timestamp,
            },
        ];

        let mut categories = HashMap::new();
        categories.insert(
            "Food".to_string(),
            vec!["(?i)coffee".to_string(), "(?i)tea".to_string()],
        );

        let messages = format_expenses_by_category(&expenses, &categories);

        // Should have 3 messages: Food category, Other category, and Total
        assert_eq!(messages.len(), 3);

        // First message should be Food category with table format
        assert!(messages[0].as_str().contains("*Food*"));
        assert!(messages[0].as_str().contains("```")); // Code block for table
        assert!(messages[0].as_str().contains("Coffee"));
        assert!(messages[0].as_str().contains("Tea"));
        assert!(messages[0].as_str().contains("Subtotal"));
        // Numbers are in code block (not escaped), formatted with 2 decimals
        assert!(messages[0].as_str().contains("5.50"));
        assert!(messages[0].as_str().contains("3.00"));
        assert!(messages[0].as_str().contains("8.50"));

        // Second message should be Other category with table format
        assert!(messages[1].as_str().contains("*Other*"));
        assert!(messages[1].as_str().contains("```")); // Code block for table
        assert!(messages[1].as_str().contains("Groceries"));
        assert!(messages[1].as_str().contains("Subtotal"));
        assert!(messages[1].as_str().contains("25.00"));

        // Third message should be Total with category breakdown in table format
        let total_msg = messages[2].as_str();
        assert!(total_msg.contains("```")); // Should be in code block
        assert!(total_msg.contains("Food"));
        assert!(total_msg.contains("8.50")); // Formatted with 2 decimal places
        assert!(total_msg.contains("Other"));
        assert!(total_msg.contains("25.00"));
        assert!(total_msg.contains("---")); // Separator line
        assert!(total_msg.contains("Total"));
        assert!(total_msg.contains("33.50"));
    }

    #[test]
    fn test_format_expenses_by_category_empty_returns_single_message() {
        let expenses = vec![];
        let categories = HashMap::new();

        let messages = format_expenses_by_category(&expenses, &categories);

        // Should return single message with "No expenses recorded yet"
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].as_str(), "No expenses recorded yet\\.");
    }

    #[test]
    fn test_format_expenses_by_category_total_message_format() {
        // Test that the total message includes category breakdowns
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expenses = vec![
            Expense {
                description: "Coffee".to_string(),
                amount: 10.0,
                timestamp,
            },
            Expense {
                description: "Bus ticket".to_string(),
                amount: 5.0,
                timestamp,
            },
            Expense {
                description: "Groceries".to_string(),
                amount: 30.0,
                timestamp,
            },
        ];

        let mut categories = HashMap::new();
        categories.insert(
            "Food".to_string(),
            vec!["(?i)coffee".to_string(), "(?i)groceries".to_string()],
        );
        categories.insert(
            "Transport".to_string(),
            vec!["(?i)bus".to_string(), "(?i)taxi".to_string()],
        );

        let messages = format_expenses_by_category(&expenses, &categories);

        // Should have 4 messages: Food, Transport, Other (none), Total
        assert_eq!(messages.len(), 3); // Food, Transport, Total (no Other since all matched)

        // Last message should be the total with breakdown in table format
        let total_msg = &messages[2];
        let total_str = total_msg.as_str();

        // Should be in a code block (monospace)
        assert!(total_str.contains("```"));

        // Should contain each category with its subtotal (formatted with 2 decimals)
        assert!(total_str.contains("Food"));
        assert!(total_str.contains("40.00"));
        assert!(total_str.contains("Transport"));
        assert!(total_str.contains("5.00"));

        // Should have separator line
        assert!(total_str.contains("---"));

        // Should have total
        assert!(total_str.contains("Total"));
        assert!(total_str.contains("45.00"));
    }
}
