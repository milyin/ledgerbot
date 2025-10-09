use crate::commands::Command;
use crate::storage_traits::{
    CategoryStorageTrait, Expense, ExpenseStorageTrait, FilterPageStorageTrait,
    FilterSelectionStorageTrait, StorageTrait,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use teloxide::types::ChatId;
use teloxide::utils::markdown::escape;
use tokio::sync::Mutex;
use tokio::fs;

// Type aliases for complex storage types
type CategoryStorageData = Arc<Mutex<HashMap<ChatId, HashMap<String, Vec<String>>>>>;
type FilterSelectionStorageData = Arc<Mutex<HashMap<(ChatId, String), Vec<String>>>>;

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
    data: CategoryStorageData,
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

/// Persistent category storage that saves data to text files named by chat ID
/// Each chat's categories are stored in a separate file for lazy loading
#[derive(Clone)]
pub struct PersistentCategoryStorage {
    // Storage directory for category files
    storage_dir: PathBuf,
    // In-memory storage using CategoryStorage
    memory_storage: CategoryStorage,
    // Track which chats have been loaded from disk: ChatId -> bool
    loaded_chats: Arc<Mutex<HashMap<ChatId, bool>>>,
}

impl PersistentCategoryStorage {
    /// Create a new persistent category storage with the specified directory
    pub fn new(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            memory_storage: CategoryStorage::new(),
            loaded_chats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the file path for a chat's categories
    fn get_file_path(&self, chat_id: ChatId) -> PathBuf {
        self.storage_dir.join(format!("{}.txt", chat_id))
    }

    /// Load categories from disk for a specific chat ID
    async fn load_chat_categories(&self, chat_id: ChatId) -> HashMap<String, Vec<String>> {
        let file_path = self.get_file_path(chat_id);
        
        match fs::read_to_string(&file_path).await {
            Ok(content) => {
                self.parse_categories_from_string(&content)
            }
            Err(_) => {
                // File doesn't exist or can't be read, return empty categories
                HashMap::new()
            }
        }
    }

    /// Parse categories from text file content
    fn parse_categories_from_string(&self, content: &str) -> HashMap<String, Vec<String>> {
        let mut categories = HashMap::new();
        let mut current_category = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                // Category header
                current_category = Some(line[1..line.len()-1].to_string());
                categories.entry(current_category.clone().unwrap()).or_insert_with(Vec::new);
            } else if let Some(ref category) = current_category {
                // Filter pattern
                if let Some(patterns) = categories.get_mut(category) {
                    patterns.push(line.to_string());
                }
            }
        }

        categories
    }

    /// Save categories to disk for a specific chat ID
    async fn save_chat_categories(&self, chat_id: ChatId, categories: &HashMap<String, Vec<String>>) -> Result<(), std::io::Error> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&self.storage_dir).await?;

        let file_path = self.get_file_path(chat_id);
        let content = self.format_categories_to_string(categories);
        
        fs::write(&file_path, content).await
    }

    /// Format categories to text file content
    fn format_categories_to_string(&self, categories: &HashMap<String, Vec<String>>) -> String {
        let mut content = String::new();
        
        for (category_name, patterns) in categories {
            content.push_str(&format!("[{}]\n", category_name));
            for pattern in patterns {
                content.push_str(&format!("{}\n", pattern));
            }
            content.push('\n'); // Empty line between categories
        }
        
        content
    }

    /// Ensure categories are loaded for a chat ID (lazy loading)
    async fn ensure_loaded(&self, chat_id: ChatId) -> HashMap<String, Vec<String>> {
        let loaded_guard = self.loaded_chats.lock().await;
        
        if loaded_guard.get(&chat_id).copied().unwrap_or(false) {
            // Already loaded, get from memory storage
            drop(loaded_guard);
            return self.memory_storage.get_chat_categories(chat_id).await;
        }

        // Not loaded yet, load from disk
        drop(loaded_guard); // Release lock while doing I/O
        let categories = self.load_chat_categories(chat_id).await;
        
        // Store in memory storage
        for (category_name, patterns) in &categories {
            for pattern in patterns {
                self.memory_storage.add_category_filter(
                    chat_id, 
                    category_name.clone(), 
                    pattern.clone()
                ).await;
            }
        }
        
        // Mark as loaded
        let mut loaded_guard = self.loaded_chats.lock().await;
        loaded_guard.insert(chat_id, true);
        
        categories
    }
}

/// Implement CategoryStorageTrait for PersistentCategoryStorage
#[async_trait::async_trait]
impl CategoryStorageTrait for PersistentCategoryStorage {
    async fn get_chat_categories(&self, chat_id: ChatId) -> HashMap<String, Vec<String>> {
        self.ensure_loaded(chat_id).await;
        self.memory_storage.get_chat_categories(chat_id).await
    }

    async fn add_category(&self, chat_id: ChatId, category_name: String) -> Result<(), String> {
        self.ensure_loaded(chat_id).await;
        let result = self.memory_storage.add_category(chat_id, category_name.clone()).await;
        
        if result.is_ok() {
            // Save updated categories to disk
            let categories = self.memory_storage.get_chat_categories(chat_id).await;
            let _ = self.save_chat_categories(chat_id, &categories).await;
        }
        
        result
    }

    async fn add_category_filter(
        &self,
        chat_id: ChatId,
        category_name: String,
        regex_pattern: String,
    ) {
        self.ensure_loaded(chat_id).await;
        self.memory_storage.add_category_filter(chat_id, category_name, regex_pattern).await;
        
        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await;
        let _ = self.save_chat_categories(chat_id, &categories).await;
    }

    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) {
        self.ensure_loaded(chat_id).await;
        self.memory_storage.remove_category_filter(chat_id, category_name, regex_pattern).await;
        
        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await;
        let _ = self.save_chat_categories(chat_id, &categories).await;
    }

    async fn remove_category(&self, chat_id: ChatId, category_name: &str) {
        self.ensure_loaded(chat_id).await;
        self.memory_storage.remove_category(chat_id, category_name).await;
        
        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await;
        let _ = self.save_chat_categories(chat_id, &categories).await;
    }

    async fn clear_chat_categories(&self, chat_id: ChatId) {
        self.ensure_loaded(chat_id).await;
        self.memory_storage.clear_chat_categories(chat_id).await;
        
        // Save empty categories to disk (creates empty file)
        let categories = HashMap::new();
        let _ = self.save_chat_categories(chat_id, &categories).await;
    }
}

/// Storage for temporary filter word selections during filter creation
/// Maps (ChatId, CategoryName) to selected words
#[derive(Clone)]
pub struct FilterSelectionStorage {
    data: FilterSelectionStorageData,
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
    /// Create a new storage with all storage types initialized (in-memory)
    pub fn new() -> Self {
        Self {
            expenses: Arc::new(ExpenseStorage::new()),
            categories: Arc::new(CategoryStorage::new()),
            filter_selection: Arc::new(FilterSelectionStorage::new()),
            filter_page: Arc::new(FilterPageStorage::new()),
        }
    }

    /// Builder-like method to configure category storage
    /// Replaces the category storage with the provided implementation
    pub fn categories_storage(mut self, storage: impl CategoryStorageTrait + 'static) -> Self {
        self.categories = Arc::new(storage);
        self
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
