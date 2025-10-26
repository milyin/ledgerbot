use std::collections::HashMap;

use yoroolbot::{markdown::MarkdownString, markdown_format, markdown_string};

use crate::{parser::format_timestamp, storage_traits::Expense};

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

    // Add final total message with category breakdown
    if !messages.is_empty() {
        let mut total_message = MarkdownString::new();

        for (category_name, subtotal) in category_subtotals {
            let line = markdown_format!("*{}*: {}\n", &*category_name, subtotal);
            total_message = total_message + line;
        }

        let total_line = markdown_format!("*Total: {}*", total);
        total_message = total_message + total_line;

        messages.push(total_message);
    }

    messages
}

/// Format expenses as a readable list with total, grouped by categories
/// (Single message version - kept for testing and backward compatibility)
#[allow(dead_code)]
pub fn format_expenses_list(
    expenses: &[Expense],
    categories: &HashMap<String, Vec<String>>,
) -> MarkdownString {
    if expenses.is_empty() {
        return markdown_string!("No expenses recorded yet\\.");
    }

    let mut result = MarkdownString::new();
    let mut total = 0.0;

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

    // Display categorized expenses
    for category_name in category_names {
        if let Some(items) = categorized.get(&category_name) {
            let (section, category_total) = format_category_section(&category_name, items);
            result = result + section;
            total += category_total;
        }
    }

    // Display uncategorized expenses
    if !uncategorized.is_empty() {
        let (section, category_total) = format_category_section("Other", &uncategorized);
        result = result + section;
        total += category_total;
    }

    let total_line = markdown_format!("*Total: {}*", total);
    result + total_line
}

/// Helper function to format a single category as a standalone message
fn format_category_message(category_name: &str, expenses: &[Expense]) -> (MarkdownString, f64) {
    let mut category_total = 0.0;
    let mut section = markdown_format!("*{}*:\n", category_name);

    for expense in expenses {
        let date_str = format_timestamp(expense.timestamp);
        let expense_line = markdown_format!(
            "  • {} {} {}\n",
            date_str,
            &*expense.description,
            expense.amount
        );
        section = section + expense_line;
        category_total += expense.amount;
    }

    let subtotal_line = markdown_format!("  *Subtotal: {}*", category_total);
    section = section + subtotal_line;

    (section, category_total)
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
    fn test_format_expenses_list_returns_markdown_string() {
        // Test that the function returns a MarkdownString
        let expenses = vec![Expense {
            description: "Test expense".to_string(),
            amount: 25.50,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }];
        let categories = HashMap::new();

        let result = format_expenses_list(&expenses, &categories);

        // The function should return a MarkdownString
        // We can verify the type by calling methods that are specific to MarkdownString
        assert!(!result.as_str().is_empty());
        assert!(result.as_str().contains("Test expense"));
        assert!(result.as_str().contains("25")); // Check for the amount
        assert!(result.as_str().contains("*Total:")); // Check for the total line
    }

    #[test]
    fn test_format_expenses_list_empty_returns_markdown_string() {
        let expenses = vec![];
        let categories = HashMap::new();

        let result = format_expenses_list(&expenses, &categories);

        // Should return the "No expenses recorded yet" message as MarkdownString
        assert_eq!(result.as_str(), "No expenses recorded yet\\.");
    }

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
        categories.insert("Food".to_string(), vec!["(?i)coffee".to_string(), "(?i)tea".to_string()]);

        let messages = format_expenses_by_category(&expenses, &categories);

        // Should have 3 messages: Food category, Other category, and Total
        assert_eq!(messages.len(), 3);

        // First message should be Food category
        assert!(messages[0].as_str().contains("*Food*"));
        assert!(messages[0].as_str().contains("Coffee"));
        assert!(messages[0].as_str().contains("Tea"));
        assert!(messages[0].as_str().contains("Subtotal"));
        // Numbers are escaped in MarkdownV2 format
        assert!(messages[0].as_str().contains("8\\.5"));

        // Second message should be Other category
        assert!(messages[1].as_str().contains("*Other*"));
        assert!(messages[1].as_str().contains("Groceries"));
        assert!(messages[1].as_str().contains("Subtotal"));
        assert!(messages[1].as_str().contains("25"));

        // Third message should be Total with category breakdown
        assert!(messages[2].as_str().contains("*Food*: 8\\.5"));
        assert!(messages[2].as_str().contains("*Other*: 25"));
        assert!(messages[2].as_str().contains("*Total: 33\\.5*"));
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
        categories.insert("Food".to_string(), vec!["(?i)coffee".to_string(), "(?i)groceries".to_string()]);
        categories.insert("Transport".to_string(), vec!["(?i)bus".to_string(), "(?i)taxi".to_string()]);

        let messages = format_expenses_by_category(&expenses, &categories);

        // Should have 4 messages: Food, Transport, Other (none), Total
        assert_eq!(messages.len(), 3); // Food, Transport, Total (no Other since all matched)

        // Last message should be the total with breakdown
        let total_msg = &messages[2];

        // Should contain each category with its subtotal
        assert!(total_msg.as_str().contains("*Food*: 40"));
        assert!(total_msg.as_str().contains("*Transport*: 5"));
        assert!(total_msg.as_str().contains("*Total: 45*"));
    }
}
