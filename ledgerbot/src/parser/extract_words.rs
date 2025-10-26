use std::collections::HashMap;

use crate::storage_traits::Expense;

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
    use std::collections::HashMap;

    use crate::{parser::extract_words::extract_words, storage_traits::Expense};

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
}
