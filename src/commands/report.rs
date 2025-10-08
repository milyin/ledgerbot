use std::collections::HashMap;

use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::{Requester, ResponseResult},
    types::Message,
    utils::markdown::escape,
};

use crate::{
    parser::format_timestamp,
    storage::Storage,
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
    storage: Storage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = storage.get_chat_expenses(chat_id).await;
    let chat_categories = storage.get_chat_categories(chat_id).await;

    // Check for category conflicts before generating report
    if let Err(conflict_message) = check_category_conflicts(&chat_expenses, &chat_categories) {
        bot.send_message(chat_id, conflict_message)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        return Ok(());
    }

    let expenses_list = format_expenses_list(&chat_expenses, &chat_categories);

    bot.send_message(chat_id, expenses_list)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

/// Format expenses as a readable list with total, grouped by categories
fn format_expenses_list(expenses: &[Expense], categories: &HashMap<String, Vec<String>>) -> String {
    if expenses.is_empty() {
        return "No expenses recorded yet.".to_string();
    }

    let mut result = String::new();
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
            total += format_category_section(&mut result, &category_name, items);
        }
    }

    // Display uncategorized expenses
    if !uncategorized.is_empty() {
        total += format_category_section(&mut result, "Other", &uncategorized);
    }

    result.push_str(&format!("*Total: {}*", escape(&total.to_string())));
    result
}

/// Helper function to format a single category section with its expenses
fn format_category_section(result: &mut String, category_name: &str, expenses: &[Expense]) -> f64 {
    let mut category_total = 0.0;
    result.push_str(&format!("*{}*:\n", escape(category_name)));

    for expense in expenses {
        let date_str = format_timestamp(expense.timestamp);
        result.push_str(&format!(
            "  • {} {} {}\n",
            escape(&date_str),
            escape(&expense.description),
            escape(&expense.amount.to_string()),
        ));
        category_total += expense.amount;
    }

    result.push_str(&format!(
        "  *Subtotal: {}*\n\n",
        escape(&category_total.to_string())
    ));

    category_total
}
