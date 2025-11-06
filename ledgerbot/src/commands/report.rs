use std::collections::HashMap;

use yoroolbot::{
    markdown::MarkdownString, markdown_format, markdown_string,
};

use crate::{
    storages::{Category, Expense, ExpensePeriod},
    utils::format_timestamp,
};

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
            let date_str = format_timestamp(conflict.expense.timestamp());
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
    category: &Category,
    all_expenses: &'a [Expense],
    categories: &HashMap<String, Vec<String>>,
) -> Vec<&'a Expense> {
    if category.is_other() {
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
                !category_matchers
                    .iter()
                    .any(|(_, regexes)| regexes.iter().any(|re| re.is_match(&expense.description)))
            })
            .collect()
    } else {
        // Specific category: expenses matching this category's filters
        let patterns = categories.get(category.as_str());
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
        let date_str = format_timestamp(expense.timestamp());

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

/// Format category comparison showing two periods side by side
/// Returns a tuple of (formatted message, list of categories found)
pub fn format_category_comparison(
    reference_expenses: Option<&[Expense]>,
    current_expenses: &[Expense],
    categories: &HashMap<String, Vec<String>>,
    reference_period: Option<&ExpensePeriod>,
    current_period: &ExpensePeriod,
) -> (MarkdownString, Vec<Category>) {
    // Check if we have a reference period to compare with
    let has_reference = reference_expenses.is_some() && reference_period.is_some();
    let reference_expenses = reference_expenses.unwrap_or(&[]);

    if current_expenses.is_empty() && reference_expenses.is_empty() {
        return (
            if let Some(ref_period) = reference_period {
                markdown_format!(
                    "No expenses recorded for periods *{}* and *{}*\\.",
                    ref_period.to_string(),
                    current_period.to_string()
                )
            } else {
                markdown_format!(
                    "No expenses recorded for period *{}*\\.",
                    current_period.to_string()
                )
            },
            Vec::new(),
        );
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

    // Helper function to group expenses by category
    let group_expenses = |expenses: &[Expense]| -> (HashMap<String, Vec<Expense>>, Vec<Expense>) {
        let mut categorized: HashMap<String, Vec<Expense>> = HashMap::new();
        let mut uncategorized: Vec<Expense> = Vec::new();

        for expense in expenses.iter() {
            let mut matched = false;
            for (category_name, regexes) in &category_matchers {
                if regexes.iter().any(|re| re.is_match(&expense.description)) {
                    categorized
                        .entry(category_name.clone())
                        .or_default()
                        .push(expense.clone());
                    matched = true;
                    break;
                }
            }
            if !matched {
                uncategorized.push(expense.clone());
            }
        }
        (categorized, uncategorized)
    };

    let (ref_categorized, ref_uncategorized) = group_expenses(reference_expenses);
    let (cur_categorized, cur_uncategorized) = group_expenses(current_expenses);

    // Collect all category names
    let mut category_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    category_names.extend(ref_categorized.keys().cloned());
    category_names.extend(cur_categorized.keys().cloned());
    let mut category_names: Vec<String> = category_names.into_iter().collect();
    category_names.sort();

    // Calculate totals for each category
    let mut category_subtotals: Vec<(Category, f64, f64)> = Vec::new();
    let mut ref_total = 0.0;
    let mut cur_total = 0.0;

    for category_name in &category_names {
        let ref_amount: f64 = ref_categorized
            .get(category_name)
            .map(|items| items.iter().map(|e| e.amount).sum())
            .unwrap_or(0.0);
        let cur_amount: f64 = cur_categorized
            .get(category_name)
            .map(|items| items.iter().map(|e| e.amount).sum())
            .unwrap_or(0.0);

        if let Ok(category) = Category::from_string(category_name) {
            category_subtotals.push((category, ref_amount, cur_amount));
            ref_total += ref_amount;
            cur_total += cur_amount;
        }
    }

    // Add "Other" category if either period has uncategorized expenses
    if !ref_uncategorized.is_empty() || !cur_uncategorized.is_empty() {
        let ref_amount: f64 = ref_uncategorized.iter().map(|e| e.amount).sum();
        let cur_amount: f64 = cur_uncategorized.iter().map(|e| e.amount).sum();
        category_subtotals.push((Category::Other, ref_amount, cur_amount));
        ref_total += ref_amount;
        cur_total += cur_amount;
    }

    // Build comparison table
    let max_name_len = category_subtotals
        .iter()
        .map(|(category, _, _)| category.as_str().chars().count())
        .max()
        .unwrap_or(0)
        .max(5);

    let mut table_lines = Vec::new();

    if has_reference {
        // Two-column comparison table
        let header_label = format!("{:<width$}", "", width = max_name_len);
        let ref_header = format!("{:>7}", reference_period.unwrap());
        let cur_header = format!("{:>7}", current_period);
        let header_line = format!("{}  {}  {}", header_label, ref_header, cur_header);
        let separation_line = "-".repeat(header_line.len());
        table_lines.push(header_line);
        table_lines.push(separation_line.clone());

        // Add each category row with both amounts
        for (category, ref_amount, cur_amount) in &category_subtotals {
            let padded_name = format!("{:<width$}", category.as_str(), width = max_name_len);
            let ref_str = format!("{:>7.2}", ref_amount);
            let cur_str = format!("{:>7.2}", cur_amount);
            table_lines.push(format!("{}  {}  {}", padded_name, ref_str, cur_str));
        }

        // Add separator line
        table_lines.push(separation_line.clone());

        // Add total row
        let total_label = format!("{:<width$}", "Total", width = max_name_len);
        let ref_total_str = format!("{:>7.2}", ref_total);
        let cur_total_str = format!("{:>7.2}", cur_total);
        table_lines.push(format!(
            "{}  {}  {}",
            total_label, ref_total_str, cur_total_str
        ));
    } else {
        // Single-column table
        let header_label = format!("{:<width$}", "", width = max_name_len);
        let cur_header = format!("{:>7}", current_period);
        let header_line = format!("{}  {}", header_label, cur_header);
        let separation_line = "-".repeat(header_line.len());
        table_lines.push(header_line);
        table_lines.push(separation_line.clone());

        // Add each category row with current amount only
        for (category, _, cur_amount) in &category_subtotals {
            let padded_name = format!("{:<width$}", category.as_str(), width = max_name_len);
            let cur_str = format!("{:>7.2}", cur_amount);
            table_lines.push(format!("{}  {}", padded_name, cur_str));
        }

        // Add separator line
        table_lines.push(separation_line.clone());

        // Add total row
        let total_label = format!("{:<width$}", "Total", width = max_name_len);
        let cur_total_str = format!("{:>7.2}", cur_total);
        table_lines.push(format!(
            "{}  {}",
            total_label, cur_total_str
        ));
    }

    // Join all lines and use @code modifier to wrap in code block
    let table_content = table_lines.join("\n");
    let message = if has_reference {
        markdown_format!(
            "📊 Expense summary for period *{}* with reference period {}\n\
             {}",
            current_period.to_string(),
            reference_period.unwrap().to_string(),
            @code table_content,
        )
    } else {
        markdown_format!(
            "📊 Expense summary for period *{}*\n\
             {}",
            current_period.to_string(),
            @code table_content,
        )
    };

    // Extract categories from subtotals
    let found_categories: Vec<Category> = category_subtotals
        .into_iter()
        .map(|(category, _, _)| category)
        .collect();

    (message, found_categories)
}
