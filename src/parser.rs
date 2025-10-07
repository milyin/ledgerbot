use crate::commands::Command;
use regex::Regex;
use std::collections::HashMap;
use teloxide::utils::command::BotCommands;

/// Parse expense lines and commands from a message text
/// Returns a tuple of (expenses, commands) where:
/// - expenses: vector of (description, amount, timestamp) tuples
/// - commands: vector of parsed Command enums
///
/// If bot_name is provided, lines starting with the bot name will have it stripped
/// timestamp is the Unix timestamp of the message date
pub fn parse_expenses(
    text: &str,
    bot_name: Option<&str>,
    timestamp: i64,
) -> (Vec<(String, f64, i64)>, Vec<Command>) {
    let mut expenses = Vec::new();
    let mut commands = Vec::new();

    // Regex pattern to match "<date> <text> <number>"
    // Date format: YYYY-MM-DD
    let re_with_date = Regex::new(r"^(\d{4}-\d{2}-\d{2})\s+(.+?)\s+(\d+(?:\.\d+)?)$").unwrap();

    // Regex pattern to match "<any text> <number>"
    // This captures text followed by a space and then a number (integer or decimal)
    let re = Regex::new(r"^(.+?)\s+(\d+(?:\.\d+)?)$").unwrap();

    for line in text.lines() {
        let mut line = line.trim();
        if line.is_empty() {
            continue;
        }

        // If leading word in the line is bot name or emoji, remove it
        // This allows commands like "@botname /help" or "📋 /report
        // or "🗑️ /clear" to be recognized as commands

        // Remove emoji prefix (simple heuristic: non-alphanumeric and non-syntactic char)
        if let Some(first_word) = line.split_whitespace().next() {
            // Check if first word is an emoji (simple heuristic: non-alphanumeric and non-syntactic char)
            if first_word
                .chars()
                .all(|c| !c.is_alphanumeric() && !c.is_ascii_punctuation())
            {
                line = line[first_word.len()..].trim_start();
            }
        }

        // Remove bot name prefix if present (case-insensitive)
        if let Some(name) = bot_name {
            let bot_name_lower = name.to_lowercase();
            let line_lower = line.to_lowercase();

            // Try to match @botname or botname at the start
            if line_lower.starts_with(&format!("@{}", bot_name_lower)) {
                line = line[name.len() + 1..].trim_start();
            } else if line_lower.starts_with(&bot_name_lower) {
                line = line[name.len()..].trim_start();
            }
        }

        // Collect lines starting with '/' as commands and parse them
        if line.starts_with('/') {
            // Command::parse expects to parse from the start of text and will consume
            // everything after the command. To parse line-by-line, we call it with just
            // this line, which will parse only this command.
            match Command::parse(line, bot_name.unwrap_or("")) {
                Ok(cmd) => {
                    commands.push(cmd);
                }
                Err(e) => {
                    // output error to the chat
                    crate::commands::output_error(bot.clone(), msg.clone(), e).await?;
                }
            }
            continue;
        }

        // Try to match pattern with date first: <date> <text> <sum>
        if let Some(captures) = re_with_date.captures(line) {
            let date_str = captures[1].trim();
            let description = captures[2].trim().to_string();
            if let Ok(amount) = captures[3].parse::<f64>() {
                // Parse the date and convert to timestamp
                if let Some(parsed_timestamp) = parse_date_to_timestamp(date_str) {
                    expenses.push((description, amount, parsed_timestamp));
                } else {
                    // If date parsing fails, use message timestamp
                    expenses.push((description, amount, timestamp));
                }
            }
        // If no date pattern matches, try pattern without date: <text> <sum>
        } else if let Some(captures) = re.captures(line) {
            let description = captures[1].trim().to_string();
            if let Ok(amount) = captures[2].parse::<f64>() {
                expenses.push((description, amount, timestamp));
            }
        }
    }

    (expenses, commands)
}

/// Parse date string to Unix timestamp
/// Supports format: YYYY-MM-DD
fn parse_date_to_timestamp(date_str: &str) -> Option<i64> {
    use chrono::NaiveDate;

    // Parse YYYY-MM-DD format
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}

/// Format expenses as a chronological list without category grouping
/// Output format: "date description price"
pub fn format_expenses_chronological(expenses: &HashMap<String, (f64, i64)>) -> String {
    if expenses.is_empty() {
        return "No expenses recorded yet.".to_string();
    }

    // Convert HashMap to Vec for sorting
    let mut expense_vec: Vec<(&String, &(f64, i64))> = expenses.iter().collect();

    // Sort by timestamp (chronological order)
    expense_vec.sort_by_key(|(_, (_, timestamp))| timestamp);

    let mut result = String::new();

    for (description, (amount, timestamp)) in expense_vec {
        let date_str = format_timestamp(*timestamp);
        result.push_str(&format!("{} {} {:.2}\n", date_str, description, amount));
    }

    result
}

/// Format expenses as a readable list with total, grouped by categories
pub fn format_expenses_list(
    expenses: &HashMap<String, (f64, i64)>,
    categories: &HashMap<String, Vec<String>>,
) -> String {
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
    let mut categorized: HashMap<String, Vec<(String, f64, i64)>> = HashMap::new();
    let mut uncategorized: Vec<(String, f64, i64)> = Vec::new();

    for (description, (amount, timestamp)) in expenses.iter() {
        let mut matched = false;

        // Try to match against each category
        for (category_name, regexes) in &category_matchers {
            // Check if description matches any of the patterns in this category
            if regexes.iter().any(|re| re.is_match(description)) {
                categorized.entry(category_name.clone()).or_default().push((
                    description.clone(),
                    *amount,
                    *timestamp,
                ));
                matched = true;
                break; // Each expense goes into first matching category
            }
        }

        if !matched {
            uncategorized.push((description.clone(), *amount, *timestamp));
        }
    }

    // Sort category names for consistent output
    let mut category_names: Vec<String> = categorized.keys().cloned().collect();
    category_names.sort();

    // Display categorized expenses
    for category_name in category_names {
        if let Some(items) = categorized.get(&category_name) {
            let mut category_total = 0.0;
            result.push_str(&format!("{}:\n", category_name));

            for (description, amount, timestamp) in items {
                let date_str = format_timestamp(*timestamp);
                result.push_str(&format!(
                    "  • {} - {:.2} ({})\n",
                    description, amount, date_str
                ));
                category_total += amount;
                total += amount;
            }

            result.push_str(&format!("  Subtotal: {:.2}_\n\n", category_total));
        }
    }

    // Display uncategorized expenses
    if !uncategorized.is_empty() {
        let mut uncategorized_total = 0.0;
        result.push_str("Other:\n");

        for (description, amount, timestamp) in uncategorized {
            let date_str = format_timestamp(timestamp);
            result.push_str(&format!(
                "  • {} - {:.2} ({})\n",
                description, amount, date_str
            ));
            uncategorized_total += amount;
            total += amount;
        }

        result.push_str(&format!("  Subtotal: {:.2}_\n\n", uncategorized_total));
    }

    result.push_str(&format!("Total: {:.2}", total));
    result
}

/// Format Unix timestamp to a human-readable date string
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, TimeZone, Utc};
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}

/// Extract unique words from uncategorized expenses
/// Returns a sorted vector of unique words (lowercased) from expense descriptions
/// that don't match any category patterns
pub fn extract_words(
    expenses: &HashMap<String, (f64, i64)>,
    categories: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    // Build regex matchers for each category (from all patterns)
    let category_matchers: Vec<regex::Regex> = categories
        .values()
        .flat_map(|patterns| patterns.iter())
        .filter_map(|pattern| regex::Regex::new(pattern).ok())
        .collect();

    // Collect unique words from uncategorized expenses
    let mut words = std::collections::HashSet::new();

    for description in expenses.keys() {
        // Check if this expense matches any category
        let matched = category_matchers.iter().any(|re| re.is_match(description));

        if !matched {
            // Split description into words and collect them
            for word in description.split_whitespace() {
                // Clean the word: lowercase, remove punctuation
                let cleaned = word
                    .to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();

                // Only include words that are at least 2 characters long
                if cleaned.len() >= 2 {
                    words.insert(cleaned);
                }
            }
        }
    }

    // Convert to sorted vector
    let mut result: Vec<String> = words.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expenses_with_date() {
        // Test parsing expenses with date prefix
        let text = "2024-10-05 Coffee 5.50\n2024-10-06 Lunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC (message timestamp)
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0].0, "Coffee".to_string());
        assert_eq!(expenses[0].1, 5.50);
        // The timestamp should be from the parsed date (2024-10-05), not the message timestamp
        assert_ne!(expenses[0].2, timestamp);
        assert_eq!(expenses[1].0, "Lunch".to_string());
        assert_eq!(expenses[1].1, 12.00);
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_expenses_with_different_date_formats() {
        // Test YYYY-MM-DD date format
        let text = "2024-10-05 Coffee 5.50\n2024-10-06 Tea 3.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC (message timestamp)
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0].0, "Coffee".to_string());
        assert_eq!(expenses[1].0, "Tea".to_string());
        // All timestamps should be different from message timestamp
        for expense in &expenses {
            assert_ne!(expense.2, timestamp);
        }
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_expenses_without_date() {
        // Test parsing expenses without date (should use message timestamp)
        let text = "Coffee 5.50\nLunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00, timestamp));
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_expenses_mixed_with_and_without_date() {
        // Test mixing expenses with and without dates
        let text = "2024-10-05 Coffee 5.50\nLunch 12.00\n2024-10-06 Dinner 15.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC (message timestamp)
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        assert_eq!(expenses.len(), 3);
        assert_eq!(expenses[0].0, "Coffee".to_string());
        assert_ne!(expenses[0].2, timestamp); // Should use parsed date
        assert_eq!(expenses[1].0, "Lunch".to_string());
        assert_eq!(expenses[1].2, timestamp); // Should use message timestamp
        assert_eq!(expenses[2].0, "Dinner".to_string());
        assert_ne!(expenses[2].2, timestamp); // Should use parsed date
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_expenses_with_bot_name() {
        // Test removing bot name prefix
        let text = "@testbot Coffee 5.50\ntestbot Lunch 12.00\nBus ticket 2.75";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, Some("testbot"), timestamp);

        assert_eq!(expenses.len(), 3);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00, timestamp));
        assert_eq!(expenses[2], ("Bus ticket".to_string(), 2.75, timestamp));
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_expenses_with_commands() {
        // Test that lines starting with '/' are collected as commands
        let text = "/help\nCoffee 5.50\n/report\nLunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00, timestamp));
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], Command::Help);
        assert_eq!(commands[1], Command::Report);
    }

    #[test]
    fn test_parse_expenses_mixed() {
        // Test mixed input with bot name and commands
        let text = "@mybot Coffee 5.50\n/help\nmybot Lunch 12.00\nBus ticket 2.75\n/report";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, Some("mybot"), timestamp);

        assert_eq!(expenses.len(), 3);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00, timestamp));
        assert_eq!(expenses[2], ("Bus ticket".to_string(), 2.75, timestamp));
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], Command::Help);
        assert_eq!(commands[1], Command::Report);
    }

    #[test]
    fn test_parse_expenses_case_insensitive_bot_name() {
        // Test that bot name matching is case-insensitive
        let text = "@TESTBOT Coffee 5.50\nTestBot Lunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, Some("testbot"), timestamp);

        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00, timestamp));
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_commands_with_bot_name() {
        // Test that commands work with bot name prefix
        let text = "@mybot /help\nmybot /report\n/clear";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, Some("mybot"), timestamp);

        assert_eq!(expenses.len(), 0);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], Command::Help);
        assert_eq!(commands[1], Command::Report);
        assert_eq!(commands[2], Command::Clear);
    }

    #[test]
    fn test_parse_commands_from_keyboard_buttons() {
        // Test that commands are extracted from keyboard button text like "📋 /report"
        let text = "📋 /report";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        assert_eq!(expenses.len(), 0);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], Command::Report);

        // Test multiple buttons
        let text2 = "🗑️ /clear";
        let (expenses2, commands2) = parse_expenses(text2, None, timestamp);

        assert_eq!(expenses2.len(), 0);
        assert_eq!(commands2.len(), 1);
        assert_eq!(commands2[0], Command::Clear);

        // Test with category command
        let text3 = "📂 /categories";
        let (expenses3, commands3) = parse_expenses(text3, None, timestamp);

        assert_eq!(expenses3.len(), 0);
        assert_eq!(commands3.len(), 1);
        assert_eq!(commands3[0], Command::Categories);
    }

    #[test]
    fn test_extract_words() {
        // Create test expenses
        let mut expenses = HashMap::new();
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        expenses.insert("Coffee at Starbucks".to_string(), (5.50, timestamp));
        expenses.insert("Lunch at restaurant".to_string(), (12.00, timestamp));
        expenses.insert("Bus ticket".to_string(), (2.75, timestamp));
        expenses.insert("Taxi ride".to_string(), (15.00, timestamp));

        // Create categories with patterns
        let mut categories = HashMap::new();
        let food_patterns = vec!["(?i)lunch".to_string()];
        categories.insert("Food".to_string(), food_patterns);

        // Extract words from uncategorized expenses
        let words = extract_words(&expenses, &categories);

        // "Lunch at restaurant" should be categorized as Food
        // So words should come from "Coffee at Starbucks", "Bus ticket", and "Taxi ride"
        assert!(words.contains(&"coffee".to_string()));
        assert!(words.contains(&"starbucks".to_string()));
        assert!(words.contains(&"bus".to_string()));
        assert!(words.contains(&"ticket".to_string()));
        assert!(words.contains(&"taxi".to_string()));
        assert!(words.contains(&"ride".to_string()));
        assert!(!words.contains(&"lunch".to_string())); // Should be categorized
        assert!(!words.contains(&"restaurant".to_string())); // Should be categorized
    }

    #[test]
    fn test_extract_words_empty() {
        // Test with no expenses
        let expenses = HashMap::new();
        let categories = HashMap::new();
        let words = extract_words(&expenses, &categories);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_extract_words_all_categorized() {
        // Create test expenses
        let mut expenses = HashMap::new();
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        expenses.insert("Coffee".to_string(), (5.50, timestamp));
        expenses.insert("Lunch".to_string(), (12.00, timestamp));

        // Create categories that match all expenses
        let mut categories = HashMap::new();
        let food_patterns = vec!["(?i).*".to_string()]; // Matches everything
        categories.insert("Food".to_string(), food_patterns);

        // Extract words - should be empty as all are categorized
        let words = extract_words(&expenses, &categories);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_format_expenses_chronological() {
        // Create test expenses with different timestamps
        let mut expenses = HashMap::new();
        let timestamp1 = 1609459200; // 2021-01-01 00:00:00 UTC
        let timestamp2 = 1609545600; // 2021-01-02 00:00:00 UTC
        let timestamp3 = 1609632000; // 2021-01-03 00:00:00 UTC

        expenses.insert("Lunch".to_string(), (12.00, timestamp2));
        expenses.insert("Coffee".to_string(), (5.50, timestamp1));
        expenses.insert("Dinner".to_string(), (25.00, timestamp3));

        let result = format_expenses_chronological(&expenses);

        // Check that expenses are listed in chronological order
        // Function returns plain format: "date description amount"
        assert!(result.contains("2021-01-01 Coffee 5.50"));
        assert!(result.contains("2021-01-02 Lunch 12.00"));
        assert!(result.contains("2021-01-03 Dinner 25.00"));

        // Verify chronological order by checking positions
        let coffee_pos = result.find("Coffee").unwrap();
        let lunch_pos = result.find("Lunch").unwrap();
        let dinner_pos = result.find("Dinner").unwrap();
        assert!(coffee_pos < lunch_pos);
        assert!(lunch_pos < dinner_pos);
    }

    #[test]
    fn test_format_expenses_chronological_empty() {
        // Test with no expenses
        let expenses = HashMap::new();
        let result = format_expenses_chronological(&expenses);
        assert_eq!(result, "No expenses recorded yet.");
    }

    #[test]
    fn test_parse_expenses_all_available_commands() {
        // Test that all available commands can be extracted from text
        // This includes both commands WITHOUT parameters and commands WITH parameters:
        //
        // Commands WITHOUT parameters:
        //   /start, /help, /list, /report, /clear, /categories, /clear_categories
        //
        // Commands WITH parameters:
        //   /add_category <name>
        //   /add_filter <category> <pattern>
        //   /remove_category <name>
        //   /remove_filter <category> <pattern>
        let text = "\
            /start\n\
            /help\n\
            /list\n\
            /report\n\
            /clear\n\
            /categories\n\
            /clear_categories\n\
            /add_category Food\n\
            /add_filter Food (?i)lunch\n\
            /remove_category Transport\n\
            /remove_filter Food (?i)coffee\n\
            Coffee 5.50\n\
            /list\n\
        ";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        // Check that we extracted the expense
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));

        // Check that all commands were extracted (total 12)
        assert_eq!(commands.len(), 12);

        // Commands without parameters (7 unique + 1 duplicate)
        assert_eq!(commands[0], Command::Start);
        assert_eq!(commands[1], Command::Help);
        assert_eq!(commands[2], Command::List);
        assert_eq!(commands[3], Command::Report);
        assert_eq!(commands[4], Command::Clear);
        assert_eq!(commands[5], Command::Categories);
        assert_eq!(commands[6], Command::ClearCategories);

        // Commands with parameters (4 commands)
        assert_eq!(
            commands[7],
            Command::AddCategory {
                name: Some("Food".to_string())
            }
        );
        assert_eq!(
            commands[8],
            Command::AddFilter {
                category: Some("Food".to_string()),
                pattern: Some("(?i)lunch".to_string())
            }
        );
        assert_eq!(
            commands[9],
            Command::RemoveCategory {
                name: Some("Transport".to_string())
            }
        );
        assert_eq!(
            commands[10],
            Command::RemoveFilter {
                category: Some("Food".to_string()),
                pattern: Some("(?i)coffee".to_string())
            }
        );

        // Duplicate command without parameters to verify repeatability
        assert_eq!(commands[11], Command::List);
    }

    #[test]
    fn test_parse_commands_with_missing_parameters() {
        // Test behavior when commands with required parameters are passed without them
        // The BotCommands parser will parse them with empty string parameters
        let text = "\
            /add_category\n\
            /add_filter\n\
            /remove_category\n\
            /remove_filter\n\
            /add_category Food\n\
            Coffee 5.50\n\
        ";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let (expenses, commands) = parse_expenses(text, None, timestamp);

        // Check that we extracted the expense
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50, timestamp));

        // Commands with missing parameters are now parsed as None
        // All commands now parse successfully with optional parameters
        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], Command::AddCategory { name: None });
        assert_eq!(
            commands[1],
            Command::AddFilter {
                category: None,
                pattern: None
            }
        );
        assert_eq!(commands[2], Command::RemoveCategory { name: None });
        assert_eq!(
            commands[3],
            Command::RemoveFilter {
                category: None,
                pattern: None
            }
        );
        assert_eq!(
            commands[4],
            Command::AddCategory {
                name: Some("Food".to_string())
            }
        );
    }
}
