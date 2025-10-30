use std::{collections::HashMap, sync::Arc};

use teloxide::types::ChatId;

use crate::{
    menus::select_word::Words,
    storages::{Expense, StorageTrait},
};

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

pub fn merge_words(existing: &[String], available: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Add existing words first
    for word in existing.iter() {
        if seen.insert(word.clone()) {
            merged.push(word.clone());
        }
    }

    // Then add available words
    for word in available.iter() {
        if seen.insert(word.clone()) {
            merged.push(word.clone());
        }
    }

    merged
}

pub async fn extract_and_merge_words(
    storage: &Arc<dyn StorageTrait>,
    chat_id: ChatId,
    words: Option<Words>,
) -> Words {
    let expenses = storage
        .clone()
        .as_expense_storage()
        .get_chat_expenses(chat_id)
        .await;
    let categories = storage
        .clone()
        .as_category_storage()
        .get_chat_categories(chat_id)
        .await
        .unwrap_or_default();

    // Extract words from uncategorized expenses
    let available_words = extract_words(&expenses, &categories);

    let current_words: Vec<String> = words.map(|w| w.into()).unwrap_or_default();
    merge_words(&current_words, &available_words).into()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{storages::Expense, utils::extract_words::extract_words};

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
