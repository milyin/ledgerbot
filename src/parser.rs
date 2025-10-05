use regex::Regex;
use std::collections::HashMap;

/// Parse expense lines from a message text
/// Returns a vector of (description, amount) tuples
pub fn parse_expenses(text: &str) -> Vec<(String, f64)> {
    let mut expenses = Vec::new();

    // Regex pattern to match "<any text> <number>"
    // This captures text followed by a space and then a number (integer or decimal)
    let re = Regex::new(r"^(.+?)\s+(\d+(?:\.\d+)?)$").unwrap();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(captures) = re.captures(line) {
            let description = captures[1].trim().to_string();
            if let Ok(amount) = captures[2].parse::<f64>() {
                expenses.push((description, amount));
            }
        }
    }

    expenses
}

/// Format expenses as a readable list with total
pub fn format_expenses_list(expenses: &HashMap<String, f64>) -> String {
    if expenses.is_empty() {
        return "No expenses recorded yet.".to_string();
    }

    let mut result = "📊 **Current Expenses:**\n\n".to_string();
    let mut total = 0.0;

    for (description, amount) in expenses.iter() {
        result.push_str(&format!("• {} - {:.2}\n", description, amount));
        total += amount;
    }

    result.push_str(&format!("\n💰 **Total: {:.2}**", total));
    result
}
