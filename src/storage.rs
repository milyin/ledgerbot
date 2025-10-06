use std::collections::HashMap;
use std::sync::Arc;
use teloxide::types::ChatId;
use tokio::sync::Mutex;

/// Per-chat storage for expenses - each chat has its own expense HashMap
/// Maps description to (amount, unix_timestamp)
pub type ExpenseStorage = Arc<Mutex<HashMap<ChatId, HashMap<String, (f64, i64)>>>>;

/// Per-chat storage for categories - each chat has its own category mappings
/// Maps category name to a list of regex patterns
pub type CategoryStorage = Arc<Mutex<HashMap<ChatId, HashMap<String, Vec<String>>>>>;

/// Storage for temporary filter word selections during filter creation
/// Maps (ChatId, CategoryName) to selected words
pub type FilterSelectionStorage = Arc<Mutex<HashMap<(ChatId, String), Vec<String>>>>;

/// Get expenses for a specific chat
pub async fn get_chat_expenses(
    storage: &ExpenseStorage,
    chat_id: ChatId,
) -> HashMap<String, (f64, i64)> {
    let storage_guard = storage.lock().await;
    storage_guard.get(&chat_id).cloned().unwrap_or_default()
}

/// Add expenses to a specific chat's storage
pub async fn add_expenses(
    storage: &ExpenseStorage,
    chat_id: ChatId,
    expenses: Vec<(String, f64, i64)>,
) {
    let mut storage_guard = storage.lock().await;
    let chat_expenses = storage_guard.entry(chat_id).or_default();
    for (description, amount, timestamp) in expenses {
        chat_expenses.insert(description, (amount, timestamp));
    }
}

/// Clear all expenses for a specific chat
pub async fn clear_chat_expenses(storage: &ExpenseStorage, chat_id: ChatId) {
    let mut storage_guard = storage.lock().await;
    storage_guard.remove(&chat_id);
}

/// Create a new expense storage
pub fn create_storage() -> ExpenseStorage {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Get categories for a specific chat
pub async fn get_chat_categories(
    storage: &CategoryStorage,
    chat_id: ChatId,
) -> HashMap<String, Vec<String>> {
    let storage_guard = storage.lock().await;
    storage_guard.get(&chat_id).cloned().unwrap_or_default()
}

/// Add or update a category for a specific chat
pub async fn add_category(storage: &CategoryStorage, chat_id: ChatId, category_name: String) {
    let mut storage_guard = storage.lock().await;
    let chat_categories = storage_guard.entry(chat_id).or_default();
    chat_categories
        .entry(category_name)
        .or_insert_with(Vec::new);
}

/// Add a regex filter to an existing category
pub async fn add_category_filter(
    storage: &CategoryStorage,
    chat_id: ChatId,
    category_name: String,
    regex_pattern: String,
) {
    let mut storage_guard = storage.lock().await;
    let chat_categories = storage_guard.entry(chat_id).or_default();
    let patterns = chat_categories
        .entry(category_name)
        .or_insert_with(Vec::new);
    if !patterns.contains(&regex_pattern) {
        patterns.push(regex_pattern);
    }
}

/// Remove a regex filter from a category
pub async fn remove_category_filter(
    storage: &CategoryStorage,
    chat_id: ChatId,
    category_name: &str,
    regex_pattern: &str,
) {
    let mut storage_guard = storage.lock().await;
    if let Some(chat_categories) = storage_guard.get_mut(&chat_id)
        && let Some(patterns) = chat_categories.get_mut(category_name)
    {
        patterns.retain(|p| p != regex_pattern);
    }
}

/// Remove a category from a specific chat
pub async fn remove_category(storage: &CategoryStorage, chat_id: ChatId, category_name: &str) {
    let mut storage_guard = storage.lock().await;
    if let Some(chat_categories) = storage_guard.get_mut(&chat_id) {
        chat_categories.remove(category_name);
    }
}

/// Create a new category storage
pub fn create_category_storage() -> CategoryStorage {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Create a new filter selection storage
pub fn create_filter_selection_storage() -> FilterSelectionStorage {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Get selected words for a filter being created
pub async fn get_filter_selection(
    storage: &FilterSelectionStorage,
    chat_id: ChatId,
    category: &str,
) -> Vec<String> {
    let storage_guard = storage.lock().await;
    storage_guard
        .get(&(chat_id, category.to_string()))
        .cloned()
        .unwrap_or_default()
}

/// Set selected words for a filter being created
pub async fn set_filter_selection(
    storage: &FilterSelectionStorage,
    chat_id: ChatId,
    category: String,
    words: Vec<String>,
) {
    let mut storage_guard = storage.lock().await;
    if words.is_empty() {
        storage_guard.remove(&(chat_id, category));
    } else {
        storage_guard.insert((chat_id, category), words);
    }
}

/// Clear filter selection for a chat/category
pub async fn clear_filter_selection(
    storage: &FilterSelectionStorage,
    chat_id: ChatId,
    category: &str,
) {
    let mut storage_guard = storage.lock().await;
    storage_guard.remove(&(chat_id, category.to_string()));
}
