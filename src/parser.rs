use regex::Regex;
use std::collections::HashMap;

/// Parse expense lines and commands from a message text
/// Returns a tuple of (expenses, commands) where:
/// - expenses: vector of (description, amount) tuples
/// - commands: vector of command strings (without the leading '/')
/// 
/// If bot_name is provided, lines starting with the bot name will have it stripped
pub fn parse_expenses(text: &str, bot_name: Option<&str>) -> (Vec<(String, f64)>, Vec<String>) {
    let mut expenses = Vec::new();
    let mut commands = Vec::new();

    // Regex pattern to match "<any text> <number>"
    // This captures text followed by a space and then a number (integer or decimal)
    let re = Regex::new(r"^(.+?)\s+(\d+(?:\.\d+)?)$").unwrap();

    for line in text.lines() {
        let mut line = line.trim();
        if line.is_empty() {
            continue;
        }

        // If line starts with bot name (case-insensitive), remove it
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

        // Collect lines starting with '/' as commands
        if line.starts_with('/') {
            commands.push(line.to_string());
            continue;
        }

        if let Some(captures) = re.captures(line) {
            let description = captures[1].trim().to_string();
            if let Ok(amount) = captures[2].parse::<f64>() {
                expenses.push((description, amount));
            }
        }
    }

    (expenses, commands)
}

/// Format expenses as a readable list with total, grouped by categories
pub fn format_expenses_list(
    expenses: &HashMap<String, f64>,
    categories: &HashMap<String, String>,
) -> String {
    if expenses.is_empty() {
        return "No expenses recorded yet.".to_string();
    }

    let mut result = "📊 **Current Expenses:**\n\n".to_string();
    let mut total = 0.0;
    
    // Build regex matchers for each category
    let category_matchers: Vec<(String, regex::Regex)> = categories
        .iter()
        .filter_map(|(name, pattern)| {
            regex::Regex::new(pattern).ok().map(|re| (name.clone(), re))
        })
        .collect();
    
    // Group expenses by category
    let mut categorized: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    let mut uncategorized: Vec<(String, f64)> = Vec::new();
    
    for (description, amount) in expenses.iter() {
        let mut matched = false;
        
        // Try to match against each category
        for (category_name, regex) in &category_matchers {
            if regex.is_match(description) {
                categorized
                    .entry(category_name.clone())
                    .or_default()
                    .push((description.clone(), *amount));
                matched = true;
                break; // Each expense goes into first matching category
            }
        }
        
        if !matched {
            uncategorized.push((description.clone(), *amount));
        }
    }
    
    // Sort category names for consistent output
    let mut category_names: Vec<String> = categorized.keys().cloned().collect();
    category_names.sort();
    
    // Display categorized expenses
    for category_name in category_names {
        if let Some(items) = categorized.get(&category_name) {
            let mut category_total = 0.0;
            result.push_str(&format!("**{}:**\n", category_name));
            
            for (description, amount) in items {
                result.push_str(&format!("  • {} - {:.2}\n", description, amount));
                category_total += amount;
                total += amount;
            }
            
            result.push_str(&format!("  _Subtotal: {:.2}_\n\n", category_total));
        }
    }
    
    // Display uncategorized expenses
    if !uncategorized.is_empty() {
        let mut uncategorized_total = 0.0;
        result.push_str("**Other:**\n");
        
        for (description, amount) in uncategorized {
            result.push_str(&format!("  • {} - {:.2}\n", description, amount));
            uncategorized_total += amount;
            total += amount;
        }
        
        result.push_str(&format!("  _Subtotal: {:.2}_\n\n", uncategorized_total));
    }

    result.push_str(&format!("💰 **Total: {:.2}**", total));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expenses_with_bot_name() {
        // Test removing bot name prefix
        let text = "@testbot Coffee 5.50\ntestbot Lunch 12.00\nBus ticket 2.75";
        let (expenses, commands) = parse_expenses(text, Some("testbot"));
        
        assert_eq!(expenses.len(), 3);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00));
        assert_eq!(expenses[2], ("Bus ticket".to_string(), 2.75));
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_expenses_with_commands() {
        // Test that lines starting with '/' are collected as commands
        let text = "/help\nCoffee 5.50\n/list\nLunch 12.00";
        let (expenses, commands) = parse_expenses(text, None);
        
        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00));
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], "/help");
        assert_eq!(commands[1], "/list");
    }

    #[test]
    fn test_parse_expenses_mixed() {
        // Test mixed input with bot name and commands
        let text = "@mybot Coffee 5.50\n/help\nmybot Lunch 12.00\nBus ticket 2.75\n/list";
        let (expenses, commands) = parse_expenses(text, Some("mybot"));
        
        assert_eq!(expenses.len(), 3);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00));
        assert_eq!(expenses[2], ("Bus ticket".to_string(), 2.75));
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], "/help");
        assert_eq!(commands[1], "/list");
    }

    #[test]
    fn test_parse_expenses_case_insensitive_bot_name() {
        // Test that bot name matching is case-insensitive
        let text = "@TESTBOT Coffee 5.50\nTestBot Lunch 12.00";
        let (expenses, commands) = parse_expenses(text, Some("testbot"));
        
        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0], ("Coffee".to_string(), 5.50));
        assert_eq!(expenses[1], ("Lunch".to_string(), 12.00));
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_parse_commands_with_bot_name() {
        // Test that commands work with bot name prefix
        let text = "@mybot /help\nmybot /list\n/clear";
        let (expenses, commands) = parse_expenses(text, Some("mybot"));
        
        assert_eq!(expenses.len(), 0);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "/help");
        assert_eq!(commands[1], "/list");
        assert_eq!(commands[2], "/clear");
    }
}
