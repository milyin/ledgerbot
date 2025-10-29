use std::collections::HashMap;

use yoroolbot::{
    command_trait::CommandTrait,
    markdown::MarkdownString,
    markdown_format, markdown_string,
    storage::ButtonData,
};

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

/// Filter expenses for a specific category
pub fn filter_category_expenses<'a>(
    category_name: &str,
    all_expenses: &'a [Expense],
    categories: &HashMap<String, Vec<String>>,
) -> Vec<&'a Expense> {
    if category_name == "Other" {
        // "Other" category: uncategorized expenses
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

        all_expenses
            .iter()
            .filter(|expense| {
                // Check if expense doesn't match any category
                !category_matchers.iter().any(|(_, regexes)| {
                    regexes.iter().any(|re| re.is_match(&expense.description))
                })
            })
            .collect()
    } else {
        // Specific category: expenses matching this category's filters
        let patterns = categories.get(category_name);
        if let Some(patterns) = patterns {
            let regexes: Vec<regex::Regex> = patterns
                .iter()
                .filter_map(|pattern| regex::Regex::new(pattern).ok())
                .collect();

            all_expenses
                .iter()
                .filter(|expense| regexes.iter().any(|re| re.is_match(&expense.description)))
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Wrap text to a maximum width, breaking at word boundaries
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.len() <= max_width {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            // First word on the line - keep it whole even if longer than max_width
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Start new line
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Format a simple report for single category with first 30 records
pub fn format_single_category_report(
    category_name: &str,
    expenses: &[&Expense],
) -> MarkdownString {
    if expenses.is_empty() {
        return markdown_format!("*{}*: No expenses in this category\\.", category_name);
    }

    // Take first 30 records
    let records_to_show: Vec<&Expense> = expenses.iter().take(30).copied().collect();

    // Find maximum amount width for alignment
    let max_amount_width = records_to_show
        .iter()
        .map(|e| format!("{:.2}", e.amount).len())
        .max()
        .unwrap_or(0);

    const DESCRIPTION_WIDTH: usize = 20;

    // Build simple text report, skipping repeating dates
    let mut report_lines = Vec::new();
    let mut last_date: Option<String> = None;

    for expense in &records_to_show {
        let date_str = format_timestamp(expense.timestamp);

        // Check if date is same as previous
        let date_field = if last_date.as_ref() == Some(&date_str.as_str().to_string()) {
            // Skip repeating date - use spaces instead
            " ".repeat(10) // Date is always 10 characters (YYYY-MM-DD)
        } else {
            // New date, show it and remember
            last_date = Some(date_str.as_str().to_string());
            date_str.as_str().to_string()
        };

        // Wrap description to max width
        let description_lines = wrap_text(&expense.description, DESCRIPTION_WIDTH);

        // Format with aligned amount after description
        let amount_str = format!("{:>width$.2}", expense.amount, width = max_amount_width);

        // First line with date, description, and amount
        let first_line = format!(
            "{}  {}  {}",
            date_field,
            format!("{:<width$}", description_lines[0], width = DESCRIPTION_WIDTH),
            amount_str
        );
        report_lines.push(first_line);

        // Additional lines for wrapped description (if any)
        for desc_line in description_lines.iter().skip(1) {
            let continuation_line = format!(
                "{}  {}",
                " ".repeat(10), // Date column
                format!("{:<width$}", desc_line, width = DESCRIPTION_WIDTH)
            );
            report_lines.push(continuation_line);
        }
    }

    // Join all lines
    let report = report_lines.join("\n");

    // Wrap in code block with header
    markdown_format!("*{}*:\n{}", category_name, @code report)
}

/// Format category summary with interactive menu for category selection
pub fn format_category_summary(
    expenses: &[Expense],
    categories: &HashMap<String, Vec<String>>,
) -> (MarkdownString, Vec<Vec<ButtonData>>) {
    if expenses.is_empty() {
        return (markdown_string!("No expenses recorded yet\\."), vec![]);
    }

    // Build regex matchers for each category
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

    // Calculate totals
    let mut category_subtotals: Vec<(String, f64)> = Vec::new();
    let mut total = 0.0;

    for category_name in &category_names {
        if let Some(items) = categorized.get(category_name) {
            let category_total: f64 = items.iter().map(|e| e.amount).sum();
            category_subtotals.push((category_name.clone(), category_total));
            total += category_total;
        }
    }

    if !uncategorized.is_empty() {
        let category_total: f64 = uncategorized.iter().map(|e| e.amount).sum();
        category_subtotals.push(("Other".to_string(), category_total));
        total += category_total;
    }

    // Build summary table
    let max_name_len = category_subtotals
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0)
        .max(5); // At least as wide as "Total"

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
    let summary_message = markdown_format!("📊 *Expense Summary*\n\n{}\n\n", @code table_content);
    let summary_message = summary_message + markdown_string!("Select a category to view details:");

    // Create inline keyboard button data using Callback
    // Callback buttons execute commands directly when clicked
    // Arrange buttons in 4 columns
    let mut buttons: Vec<Vec<ButtonData>> = Vec::new();
    let mut current_row: Vec<ButtonData> = Vec::new();

    for (category_name, _) in &category_subtotals {
        let command = crate::commands::command_report::CommandReport {
            category: Some(category_name.clone()),
        };
        current_row.push(ButtonData::Callback(
            category_name.clone(),
            command.to_command_string(false),
        ));

        // Start a new row after 4 buttons
        if current_row.len() == 4 {
            buttons.push(current_row.clone());
            current_row.clear();
        }
    }

    // Add remaining buttons if any
    if !current_row.is_empty() {
        buttons.push(current_row);
    }

    (summary_message, buttons)
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
