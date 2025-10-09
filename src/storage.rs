use crate::commands::Command;
use crate::storage_traits::{
    CategoryStorageTrait, Expense, ExpenseStorageTrait, FilterPageStorageTrait,
    FilterSelectionStorageTrait, StorageTrait,
};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::types::ChatId;
use teloxide::utils::markdown::escape;
use tokio::sync::Mutex;

/// Per-chat storage for expenses - each chat has its own expense list
#[derive(Clone)]
pub struct ExpenseStorage {
    data: Arc<Mutex<HashMap<ChatId, Vec<Expense>>>>,
}

impl ExpenseStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Implement ExpenseStorageTrait for ExpenseStorage
#[async_trait::async_trait]
impl ExpenseStorageTrait for ExpenseStorage {
    async fn get_chat_expenses(&self, chat_id: ChatId) -> Vec<Expense> {
        let storage_guard = self.data.lock().await;
        storage_guard.get(&chat_id).cloned().unwrap_or_default()
    }

    async fn add_expenses(&self, chat_id: ChatId, expenses: Vec<(String, f64, i64)>) {
        let mut storage_guard = self.data.lock().await;
        let chat_expenses = storage_guard.entry(chat_id).or_default();
        for (description, amount, timestamp) in expenses {
            chat_expenses.push(Expense {
                description,
                amount,
                timestamp,
            });
        }
    }

    async fn add_expense(&self, chat_id: ChatId, description: &str, amount: f64, timestamp: i64) {
        self.add_expenses(chat_id, vec![(description.to_string(), amount, timestamp)])
            .await;
    }

    async fn clear_chat_expenses(&self, chat_id: ChatId) {
        let mut storage_guard = self.data.lock().await;
        storage_guard.remove(&chat_id);
    }
}

/// Per-chat storage for categories - each chat has its own category mappings
/// Maps category name to a list of regex patterns
#[derive(Clone)]
pub struct CategoryStorage {
    data: Arc<Mutex<HashMap<ChatId, HashMap<String, Vec<String>>>>>,
}

impl CategoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Implement CategoryStorageTrait for CategoryStorage
#[async_trait::async_trait]
impl CategoryStorageTrait for CategoryStorage {
    async fn get_chat_categories(&self, chat_id: ChatId) -> HashMap<String, Vec<String>> {
        let storage_guard = self.data.lock().await;
        storage_guard.get(&chat_id).cloned().unwrap_or_default()
    }

    async fn add_category(&self, chat_id: ChatId, category_name: String) -> Result<(), String> {
        // Acquire lock once and hold it for the entire operation to prevent race conditions
        let mut storage_guard = self.data.lock().await;
        let chat_categories = storage_guard.entry(chat_id).or_default();

        // Check if category already exists (while holding the lock)
        if chat_categories.contains_key(&category_name) {
            return Err(format!(
                "ℹ️ Category `{}` already exists. Use {} to add more patterns or {} to view all.",
                category_name,
                escape(Command::ADD_FILTER),
                escape(Command::CATEGORIES)
            ));
        }

        // Add the new category
        chat_categories.insert(category_name.clone(), Vec::new());

        Ok(())
    }

    async fn add_category_filter(
        &self,
        chat_id: ChatId,
        category_name: String,
        regex_pattern: String,
    ) {
        let mut storage_guard = self.data.lock().await;
        let chat_categories = storage_guard.entry(chat_id).or_default();
        let patterns = chat_categories
            .entry(category_name)
            .or_insert_with(Vec::new);
        if !patterns.contains(&regex_pattern) {
            patterns.push(regex_pattern);
        }
    }

    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) {
        let mut storage_guard = self.data.lock().await;
        if let Some(chat_categories) = storage_guard.get_mut(&chat_id)
            && let Some(patterns) = chat_categories.get_mut(category_name)
        {
            patterns.retain(|p| p != regex_pattern);
        }
    }

    async fn remove_category(&self, chat_id: ChatId, category_name: &str) {
        let mut storage_guard = self.data.lock().await;
        if let Some(chat_categories) = storage_guard.get_mut(&chat_id) {
            chat_categories.remove(category_name);
        }
    }

    async fn clear_chat_categories(&self, chat_id: ChatId) {
        let mut storage_guard = self.data.lock().await;
        storage_guard.remove(&chat_id);
    }
}

/// Storage for temporary filter word selections during filter creation
/// Maps (ChatId, CategoryName) to selected words
#[derive(Clone)]
pub struct FilterSelectionStorage {
    data: Arc<Mutex<HashMap<(ChatId, String), Vec<String>>>>,
}

impl FilterSelectionStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Implement FilterSelectionStorageTrait for FilterSelectionStorage
#[async_trait::async_trait]
impl FilterSelectionStorageTrait for FilterSelectionStorage {
    async fn get_filter_selection(&self, chat_id: ChatId, category: &str) -> Vec<String> {
        let storage_guard = self.data.lock().await;
        storage_guard
            .get(&(chat_id, category.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    async fn set_filter_selection(&self, chat_id: ChatId, category: String, words: Vec<String>) {
        let mut storage_guard = self.data.lock().await;
        if words.is_empty() {
            storage_guard.remove(&(chat_id, category));
        } else {
            storage_guard.insert((chat_id, category), words);
        }
    }

    async fn clear_filter_selection(&self, chat_id: ChatId, category: &str) {
        let mut storage_guard = self.data.lock().await;
        storage_guard.remove(&(chat_id, category.to_string()));
    }
}

/// Storage for page offsets during filter word browsing
/// Maps (ChatId, CategoryName) to current page offset
#[derive(Clone)]
pub struct FilterPageStorage {
    data: Arc<Mutex<HashMap<(ChatId, String), usize>>>,
}

impl FilterPageStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Implement FilterPageStorageTrait for FilterPageStorage
#[async_trait::async_trait]
impl FilterPageStorageTrait for FilterPageStorage {
    async fn get_filter_page_offset(&self, chat_id: ChatId, category: &str) -> usize {
        let storage_guard = self.data.lock().await;
        storage_guard
            .get(&(chat_id, category.to_string()))
            .copied()
            .unwrap_or(0)
    }

    async fn set_filter_page_offset(&self, chat_id: ChatId, category: String, offset: usize) {
        let mut storage_guard = self.data.lock().await;
        if offset == 0 {
            storage_guard.remove(&(chat_id, category));
        } else {
            storage_guard.insert((chat_id, category), offset);
        }
    }

    async fn clear_filter_page_offset(&self, chat_id: ChatId, category: &str) {
        let mut storage_guard = self.data.lock().await;
        storage_guard.remove(&(chat_id, category.to_string()));
    }
}

/// Main storage structure that holds all bot data
/// This is the primary storage container for the application
#[derive(Clone)]
pub struct Storage {
    expenses: Arc<dyn ExpenseStorageTrait>,
    categories: Arc<dyn CategoryStorageTrait>,
    filter_selection: Arc<dyn FilterSelectionStorageTrait>,
    filter_page: Arc<dyn FilterPageStorageTrait>,
}

impl Storage {
    /// Create a new storage with all storage types initialized
    pub fn new() -> Self {
        Self {
            expenses: Arc::new(ExpenseStorage::new()),
            categories: Arc::new(CategoryStorage::new()),
            filter_selection: Arc::new(FilterSelectionStorage::new()),
            filter_page: Arc::new(FilterPageStorage::new()),
        }
    }

    /// Replace the expense storage with a new instance
    pub fn replace_expenses(&mut self, expenses: Arc<dyn ExpenseStorageTrait>) {
        self.expenses = expenses;
    }

    /// Replace the category storage with a new instance
    pub fn replace_categories(&mut self, categories: Arc<dyn CategoryStorageTrait>) {
        self.categories = categories;
    }

    /// Replace the filter selection storage with a new instance
    pub fn replace_filter_selection(&mut self, new_filter_selection: Arc<dyn FilterSelectionStorageTrait>) {
        self.filter_selection = new_filter_selection;
    }

    /// Replace the filter page storage with a new instance
    pub fn replace_filter_page(&mut self, filter_page: Arc<dyn FilterPageStorageTrait>) {
        self.filter_page = filter_page;
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement StorageTrait for Storage to enable conversion to specific trait objects
impl StorageTrait for Storage {
    fn as_expense_storage(self: Arc<Self>) -> Arc<dyn ExpenseStorageTrait> {
        self.expenses.clone()
    }

    fn as_category_storage(self: Arc<Self>) -> Arc<dyn CategoryStorageTrait> {
        self.categories.clone()
    }

    fn as_filter_selection_storage(self: Arc<Self>) -> Arc<dyn FilterSelectionStorageTrait> {
        self.filter_selection.clone()
    }

    fn as_filter_page_storage(self: Arc<Self>) -> Arc<dyn FilterPageStorageTrait> {
        self.filter_page.clone()
    }
}
