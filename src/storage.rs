use std::collections::HashMap;
use std::sync::Arc;
use teloxide::types::ChatId;
use tokio::sync::Mutex;

/// Per-chat storage for expenses - each chat has its own expense HashMap
pub type ExpenseStorage = Arc<Mutex<HashMap<ChatId, HashMap<String, f64>>>>;

/// Per-chat storage for categories - each chat has its own category mappings
/// Maps category name to regex pattern
pub type CategoryStorage = Arc<Mutex<HashMap<ChatId, HashMap<String, String>>>>;

/// Get expenses for a specific chat
pub async fn get_chat_expenses(storage: &ExpenseStorage, chat_id: ChatId) -> HashMap<String, f64> {
    let storage_guard = storage.lock().await;
    storage_guard.get(&chat_id).cloned().unwrap_or_default()
}

/// Add expenses to a specific chat's storage
pub async fn add_expenses(
    storage: &ExpenseStorage,
    chat_id: ChatId,
    expenses: Vec<(String, f64)>,
) {
    let mut storage_guard = storage.lock().await;
    let chat_expenses = storage_guard.entry(chat_id).or_default();
    for (description, amount) in expenses {
        chat_expenses.insert(description, amount);
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
pub async fn get_chat_categories(storage: &CategoryStorage, chat_id: ChatId) -> HashMap<String, String> {
    let storage_guard = storage.lock().await;
    storage_guard.get(&chat_id).cloned().unwrap_or_default()
}

/// Add or update a category for a specific chat
pub async fn add_category(
    storage: &CategoryStorage,
    chat_id: ChatId,
    category_name: String,
    regex_pattern: String,
) {
    let mut storage_guard = storage.lock().await;
    let chat_categories = storage_guard.entry(chat_id).or_default();
    chat_categories.insert(category_name, regex_pattern);
}

/// Remove a category from a specific chat
pub async fn remove_category(
    storage: &CategoryStorage,
    chat_id: ChatId,
    category_name: &str,
) {
    let mut storage_guard = storage.lock().await;
    if let Some(chat_categories) = storage_guard.get_mut(&chat_id) {
        chat_categories.remove(category_name);
    }
}

/// Create a new category storage
pub fn create_category_storage() -> CategoryStorage {
    Arc::new(Mutex::new(HashMap::new()))
}
