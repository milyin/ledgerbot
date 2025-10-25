use std::collections::HashMap;

use chrono::{NaiveDate, TimeZone, Utc};
use teloxide::utils::command::BotCommands;

use crate::{
    commands::{command_add_expense::CommandAddExpense, command_trait::CommandTrait, Command},
    storage_traits::Expense,
};

/// Parse expense lines and commands from a message text
/// Returns a vector of Results containing either successfully parsed Commands or error messages
/// where text lines matching expense patterns are converted to Command::AddExpense variants
///
/// If bot_name is provided, lines starting with the bot name will have it stripped
/// timestamp is the Unix timestamp of the message date
pub fn parse_expenses(
    text: &str,
    bot_name: Option<&str>,
    timestamp: i64,
) -> Vec<Result<Command, String>> {
    let mut commands = Vec::new();
    let message_date = Utc.timestamp_opt(timestamp, 0).unwrap().date_naive();

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

        let command_line = if !line.starts_with('/') {
            // Convert non-command lines to /add_expense with explicit date
            // Check if line already starts with a date (YYYY-MM-DD format)
            let parts: Vec<&str> = line.split_whitespace().collect();
            let parsed_date = parts
                .first()
                .and_then(|first_word| NaiveDate::parse_from_str(first_word, "%Y-%m-%d").ok());

            let (date, description_start_idx) = if let Some(explicit_date) = parsed_date {
                // Line has explicit date: "YYYY-MM-DD description amount"
                (explicit_date, 1)
            } else {
                // Line doesn't have date: "description amount"
                (message_date, 0)
            };

            // Extract amount and description
            let amount = parts.last().and_then(|s| s.parse::<f64>().ok());
            let description_parts = &parts[description_start_idx..parts.len() - 1];
            let description = if description_parts.is_empty() {
                None
            } else {
                Some(description_parts.join(" "))
            };

            // Create command object and use to_command_string
            let cmd = CommandAddExpense {
                date: Some(date),
                description,
                amount,
            };
            cmd.to_command_string(false)
        } else {
            line.to_string()
        };

        // Parse the line as a command
        match Command::parse(&command_line, bot_name.unwrap_or("")) {
            Ok(cmd) => {
                // No longer need to fill in missing dates - CommandAddExpense always has a date
                commands.push(Ok(cmd));
            }
            Err(e) => {
                commands.push(Err(format!("❌ Failed to parse command `{}`: {}", line, e)));
            }
        }
    }

    commands
}

/// Format Unix timestamp to a human-readable date string
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, TimeZone, Utc};
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}

/// Extract unique words from uncategorized expenses
/// Returns a sorted vector of unique words (lowercased) from expense descriptions
/// that don't match any category patterns
pub fn extract_words(
    expenses: &[Expense],
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

    for expense in expenses.iter() {
        // Check if this expense matches any category
        let matched = category_matchers
            .iter()
            .any(|re| re.is_match(&expense.description));

        if !matched {
            // Split description into words and collect them
            for word in expense.description.split_whitespace() {
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
    use chrono::NaiveDate;

    use super::*;
    use crate::commands::command_add_category::CommandAddCategory;

    #[test]
    fn test_parse_expenses_with_date() {
        // Test parsing expenses with date prefix
        let text = "2024-10-05 Coffee 5.50\n2024-10-06 Lunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC (message timestamp)
        let results = parse_expenses(text, None, timestamp);

        assert_eq!(results.len(), 2);

        // Check first expense
        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2024, 10, 5).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Check second expense
        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2024, 10, 6).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );
    }

    #[test]
    fn test_parse_expenses_with_different_date_formats() {
        // Test YYYY-MM-DD date format
        let text = "2024-10-05 Coffee 5.50\n2024-10-06 Tea 3.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC (message timestamp)
        let results = parse_expenses(text, None, timestamp);

        assert_eq!(results.len(), 2);

        // Check first expense
        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2024, 10, 5).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Check second expense
        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2024, 10, 6).unwrap())
            && cmd.description == Some("Tea".to_string())
            && cmd.amount == Some(3.00))
        );
    }

    #[test]
    fn test_parse_expenses_without_date() {
        // Test parsing expenses without date (should use message timestamp)
        let text = "Coffee 5.50\nLunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, None, timestamp);

        assert_eq!(results.len(), 2);

        // Check first expense (should use message timestamp as 2021-01-01)
        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Check second expense
        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );
    }

    #[test]
    fn test_parse_expenses_mixed_with_and_without_date() {
        // Test mixing expenses with and without dates
        let text = "2024-10-05 Coffee 5.50\nLunch 12.00\n2024-10-06 Dinner 15.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC (message timestamp)
        let results = parse_expenses(text, None, timestamp);

        assert_eq!(results.len(), 3);

        // Check first expense with explicit date
        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2024, 10, 5).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Check second expense without date (should use message timestamp)
        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );

        // Check third expense with explicit date
        assert!(
            matches!(&results[2], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2024, 10, 6).unwrap())
            && cmd.description == Some("Dinner".to_string())
            && cmd.amount == Some(15.00))
        );
    }

    #[test]
    fn test_parse_expenses_with_bot_name() {
        // Test removing bot name prefix
        let text = "@testbot Coffee 5.50\ntestbot Lunch 12.00\nBus ticket 2.75";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, Some("testbot"), timestamp);

        assert_eq!(results.len(), 3);

        // Check all expenses are parsed correctly with bot name removed
        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );

        assert!(
            matches!(&results[2], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Bus ticket".to_string())
            && cmd.amount == Some(2.75))
        );
    }

    #[test]
    fn test_parse_expenses_with_commands() {
        // Test that lines starting with '/' are collected as commands
        let text = "/help\nCoffee 5.50\n/report\nLunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, None, timestamp);

        assert_eq!(results.len(), 4);

        // Check first command
        assert!(matches!(&results[0], Ok(Command::Help(_))));

        // Check first expense
        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Check second command
        assert!(matches!(&results[2], Ok(Command::Report(_))));

        // Check second expense
        assert!(
            matches!(&results[3], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );
    }

    #[test]
    fn test_parse_expenses_mixed() {
        // Test mixed input with bot name and commands
        let text = "@mybot Coffee 5.50\n/help\nmybot Lunch 12.00\nBus ticket 2.75\n/report";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, Some("mybot"), timestamp);

        assert_eq!(results.len(), 5);

        // Check first expense
        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Check first command
        assert!(matches!(&results[1], Ok(Command::Help(_))));

        // Check second expense
        assert!(
            matches!(&results[2], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );

        // Check third expense
        assert!(
            matches!(&results[3], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Bus ticket".to_string())
            && cmd.amount == Some(2.75))
        );

        // Check second command
        assert!(matches!(&results[4], Ok(Command::Report(_))));
    }

    #[test]
    fn test_parse_expenses_case_insensitive_bot_name() {
        // Test that bot name matching is case-insensitive
        let text = "@TESTBOT Coffee 5.50\nTestBot Lunch 12.00";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, Some("testbot"), timestamp);

        assert_eq!(results.len(), 2);

        assert!(
            matches!(&results[0], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        assert!(
            matches!(&results[1], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Lunch".to_string())
            && cmd.amount == Some(12.00))
        );
    }

    #[test]
    fn test_parse_commands_with_bot_name() {
        // Test that commands work with bot name prefix
        let text = "@mybot /help\nmybot /report\n/clear";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, Some("mybot"), timestamp);

        assert_eq!(results.len(), 3);

        assert!(matches!(&results[0], Ok(Command::Help(_))));
        assert!(matches!(&results[1], Ok(Command::Report(_))));
        assert!(matches!(&results[2], Ok(Command::Clear(_))));
    }

    #[test]
    fn test_parse_commands_from_keyboard_buttons() {
        // Test that commands are extracted from keyboard button text like "📋 /report"
        let text = "📋 /report";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, None, timestamp);

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], Ok(Command::Report(_))));

        // Test multiple buttons
        let text2 = "🗑️ /clear";
        let results2 = parse_expenses(text2, None, timestamp);

        assert_eq!(results2.len(), 1);
        assert!(matches!(&results2[0], Ok(Command::Clear(_))));

        // Test with category command
        let text3 = "📂 /categories";
        let results3 = parse_expenses(text3, None, timestamp);

        assert_eq!(results3.len(), 1);
        assert!(matches!(&results3[0], Ok(Command::Categories(_))));
    }

    #[test]
    fn test_extract_words() {
        // Create test expenses
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let expenses = vec![
            Expense {
                description: "Coffee at Starbucks".to_string(),
                amount: 5.50,
                timestamp,
            },
            Expense {
                description: "Lunch at restaurant".to_string(),
                amount: 12.00,
                timestamp,
            },
            Expense {
                description: "Bus ticket".to_string(),
                amount: 2.75,
                timestamp,
            },
            Expense {
                description: "Taxi ride".to_string(),
                amount: 15.00,
                timestamp,
            },
        ];

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
        let expenses = Vec::new();
        let categories = HashMap::new();
        let words = extract_words(&expenses, &categories);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_extract_words_all_categorized() {
        // Create test expenses
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let expenses = vec![
            Expense {
                description: "Coffee".to_string(),
                amount: 5.50,
                timestamp,
            },
            Expense {
                description: "Lunch".to_string(),
                amount: 12.00,
                timestamp,
            },
        ];

        // Create categories that match all expenses
        let mut categories = HashMap::new();
        let food_patterns = vec!["(?i).*".to_string()]; // Matches everything
        categories.insert("Food".to_string(), food_patterns);

        // Extract words - should be empty as all are categorized
        let words = extract_words(&expenses, &categories);
        assert_eq!(words.len(), 0);
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
        //   /remove_filter <category> <position>
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
            /remove_filter Food 0\n\
            Coffee 5.50\n\
            /list\n\
        ";
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let results = parse_expenses(text, None, timestamp);

        // Check that all commands and expense were extracted (total 13)
        assert_eq!(results.len(), 13);

        // Commands without parameters (7 unique)
        assert!(matches!(&results[0], Ok(Command::Start(_))));
        assert!(matches!(&results[1], Ok(Command::Help(_))));
        assert!(matches!(&results[2], Ok(Command::List(_))));
        assert!(matches!(&results[3], Ok(Command::Report(_))));
        assert!(matches!(&results[4], Ok(Command::Clear(_))));
        assert!(matches!(&results[5], Ok(Command::Categories(_))));
        assert!(matches!(&results[6], Ok(Command::ClearCategories(_))));

        // Commands with parameters (4 commands)
        assert!(
            matches!(&results[7], Ok(Command::AddCategory(CommandAddCategory { name }))
            if name == &Some("Food".to_string()))
        );

        assert!(
            matches!(&results[8], Ok(Command::AddFilter { category, pattern })
            if category == &Some("Food".to_string())
            && pattern == &Some("(?i)lunch".to_string()))
        );

        // assert!(matches!(&results[9], Ok(Command::RemoveCategory { name })
        //     if name == &Some("Transport".to_string())));

        assert!(
            matches!(&results[10], Ok(Command::RemoveFilter(remove_filter))
            if remove_filter.category == Some("Food".to_string())
            && remove_filter.position == Some(0))
        );

        // Check the expense
        assert!(
            matches!(&results[11], Ok(Command::AddExpense(cmd))
            if cmd.date == Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap())
            && cmd.description == Some("Coffee".to_string())
            && cmd.amount == Some(5.50))
        );

        // Duplicate command without parameters to verify repeatability
        assert!(matches!(&results[12], Ok(Command::List(_))));
    }
}
