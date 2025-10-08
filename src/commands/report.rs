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
    storage::{CategoryStorage, Expense, ExpenseStorage, get_chat_categories, get_chat_expenses},
};

/// Report all expenses grouped by categories
pub async fn report_command(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = get_chat_expenses(&storage, chat_id).await;
    let chat_categories = get_chat_categories(&category_storage, chat_id).await;
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
