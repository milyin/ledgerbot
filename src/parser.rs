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
