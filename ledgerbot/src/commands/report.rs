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
/// Uses Unicode character counting for proper width calculation
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let char_count = text.chars().count();
    if char_count <= max_width {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.chars().count();

        if current_width == 0 {
            // First word on the line - keep it whole even if longer than max_width
            current_line = word.to_string();
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            // Word fits on current line (including space separator)
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            // Start new line
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Format a simple report for single category with pagination
/// Returns only the formatted expense data (without header or total)
pub fn format_single_category_report(
    expenses: &[&Expense],
    page_number: usize,
    records_per_page: usize,
) -> String {

    if expenses.is_empty() {
        return String::new();
    }

    // Calculate page offset
    let page_offset = page_number * records_per_page;

    // Get records for current page
    let records_to_show: Vec<&Expense> = expenses
        .iter()
        .skip(page_offset)
        .take(records_per_page)
        .copied()
        .collect();

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
        // Pad description to fixed width using char count for Unicode support
        let desc_width = description_lines[0].chars().count();
        let padding = if desc_width < DESCRIPTION_WIDTH {
            " ".repeat(DESCRIPTION_WIDTH - desc_width)
        } else {
            String::new()
        };
        let first_line = format!(
            "{}  {}{}  {}",
            date_field, &description_lines[0], padding, amount_str
        );
        report_lines.push(first_line);

        // Additional lines for wrapped description (if any)
        for desc_line in description_lines.iter().skip(1) {
            let desc_width = desc_line.chars().count();
            let padding = if desc_width < DESCRIPTION_WIDTH {
                " ".repeat(DESCRIPTION_WIDTH - desc_width)
            } else {
                String::new()
            };
            let continuation_line = format!(
                "{}  {}{}",
                " ".repeat(10), // Date column
                desc_line,
                padding
            );
            report_lines.push(continuation_line);
        }
    }

    // Join all lines and return
    report_lines.join("\n")
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
            page: None,
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
    // Tests for the new report system have been removed as the implementation
    // has been completely rewritten to use interactive menus and pagination.
    // The old format_expenses_by_category function no longer exists.
    // New functionality is tested through integration tests.
}
