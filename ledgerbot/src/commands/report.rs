use std::{collections::HashMap, sync::Arc};

use teloxide::{Bot, prelude::ResponseResult, types::Message, utils::markdown::escape};
use yoroolbot::{
    markdown::{MarkdownString, MarkdownStringMessage},
    markdown_format, markdown_string,
};

use crate::{
    parser::format_timestamp,
    storage_traits::{CategoryStorageTrait, Expense, ExpenseStorageTrait},
};

/// Represents a conflict where an expense matches multiple categories
#[derive(Debug, Clone)]
struct CategoryConflict {
    expense: Expense,
    matching_categories: Vec<(String, String)>, // (category_name, matched_pattern)
}

/// Check if any expense matches multiple categories
/// Returns Err with formatted error message if conflicts are found
fn check_category_conflicts(
    expenses: &[Expense],
    categories: &HashMap<String, Vec<String>>,
) -> Result<(), String> {
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
        let mut error_message = String::from("❌ *Category Conflicts Detected*\n\n");
        error_message.push_str("The following expenses match multiple categories\\.\n");
        error_message.push_str("Please adjust your filters to avoid overlapping categories\\.\n\n");

        for conflict in conflicts {
            let date_str = format_timestamp(conflict.expense.timestamp);
            error_message.push_str(&format!(
                "📝 *Expense:* {} {} {}\n",
                escape(&date_str),
                escape(&conflict.expense.description),
                escape(&conflict.expense.amount.to_string())
            ));
            error_message.push_str("*Matching categories:*\n");
            for (category_name, pattern) in conflict.matching_categories {
                error_message.push_str(&format!(
                    "  • {} \\(filter: `{}`\\)\n",
                    escape(&category_name),
                    escape(&pattern)
                ));
            }
            error_message.push('\n');
        }

        return Err(error_message);
    }

    Ok(())
}

/// Report all expenses grouped by categories
pub async fn report_command(
    bot: Bot,
    msg: Message,
    expense_storage: Arc<dyn ExpenseStorageTrait>,
    category_storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = expense_storage.get_chat_expenses(chat_id).await;
    let chat_categories = category_storage.get_chat_categories(chat_id).await;

    // Check for category conflicts before generating report
    if let Err(conflict_message) = check_category_conflicts(&chat_expenses, &chat_categories) {
        bot.markdown_message(chat_id, None, markdown_format!("{}", conflict_message))
            .await?;
        return Ok(());
    }

    let expenses_list = format_expenses_list(&chat_expenses, &chat_categories);

    bot.markdown_message(chat_id, None, expenses_list).await?;
    Ok(())
}

/// Format expenses as a readable list with total, grouped by categories
fn format_expenses_list(
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

/// Helper function to format a single category section with its expenses
fn format_category_section(category_name: &str, expenses: &[Expense]) -> (MarkdownString, f64) {
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

    let subtotal_line = markdown_format!("  *Subtotal: {}*\n\n", category_total);
    section = section + subtotal_line;

    (section, category_total)
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
}
