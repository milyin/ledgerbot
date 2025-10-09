use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::types::ChatId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub timestamp: i64,
    pub description: String,
    pub amount: f64,
}

/// Trait for expense storage operations
#[async_trait::async_trait]
pub trait ExpenseStorageTrait: Send + Sync {
    /// Get expenses for a specific chat
    async fn get_chat_expenses(&self, chat_id: ChatId) -> Vec<Expense>;
    
    /// Add expenses to a specific chat's storage
    async fn add_expenses(&self, chat_id: ChatId, expenses: Vec<(String, f64, i64)>);
    
    /// Add a single expense
    async fn add_expense(&self, chat_id: ChatId, description: &str, amount: f64, timestamp: i64);
    
    /// Clear all expenses for a specific chat
    async fn clear_chat_expenses(&self, chat_id: ChatId);
}

/// Trait for category storage operations
#[async_trait::async_trait]
pub trait CategoryStorageTrait: Send + Sync {
    /// Get categories for a specific chat
    async fn get_chat_categories(&self, chat_id: ChatId) -> HashMap<String, Vec<String>>;
    
    /// Add a category for a specific chat
    async fn add_category(&self, chat_id: ChatId, category_name: String) -> Result<(), String>;
    
    /// Add a regex filter to an existing category
    async fn add_category_filter(&self, chat_id: ChatId, category_name: String, regex_pattern: String);
    
    /// Remove a regex filter from a category
    async fn remove_category_filter(&self, chat_id: ChatId, category_name: &str, regex_pattern: &str);
    
    /// Remove a category from a specific chat
    async fn remove_category(&self, chat_id: ChatId, category_name: &str);
    
    /// Clear all categories for a specific chat
    async fn clear_chat_categories(&self, chat_id: ChatId);
}

/// Trait for filter selection storage operations (temporary filter word selections)
#[async_trait::async_trait]
pub trait FilterSelectionStorageTrait: Send + Sync {
    /// Get selected words for a filter being created
    async fn get_filter_selection(&self, chat_id: ChatId, category: &str) -> Vec<String>;
    
    /// Set selected words for a filter being created
    async fn set_filter_selection(&self, chat_id: ChatId, category: String, words: Vec<String>);
    
    /// Clear filter selection for a chat/category
    async fn clear_filter_selection(&self, chat_id: ChatId, category: &str);
}

/// Trait for filter page storage operations (pagination during filter browsing)
#[async_trait::async_trait]
pub trait FilterPageStorageTrait: Send + Sync {
    /// Get current page offset for filter word browsing
    async fn get_filter_page_offset(&self, chat_id: ChatId, category: &str) -> usize;
    
    /// Set page offset for filter word browsing
    async fn set_filter_page_offset(&self, chat_id: ChatId, category: String, offset: usize);
    
    /// Clear page offset for filter word browsing
    async fn clear_filter_page_offset(&self, chat_id: ChatId, category: &str);
}

/// Combined storage trait that provides all storage operations
/// This trait allows converting to specific trait objects for functions that only need subset of functionality
pub trait StorageTrait: ExpenseStorageTrait + CategoryStorageTrait + FilterSelectionStorageTrait + FilterPageStorageTrait + Send + Sync {
    /// Convert to ExpenseStorageTrait trait object
    fn as_expense_storage(self: Arc<Self>) -> Arc<dyn ExpenseStorageTrait>;
    
    /// Convert to CategoryStorageTrait trait object
    fn as_category_storage(self: Arc<Self>) -> Arc<dyn CategoryStorageTrait>;
    
    /// Convert to FilterSelectionStorageTrait trait object
    fn as_filter_selection_storage(self: Arc<Self>) -> Arc<dyn FilterSelectionStorageTrait>;
    
    /// Convert to FilterPageStorageTrait trait object
    fn as_filter_page_storage(self: Arc<Self>) -> Arc<dyn FilterPageStorageTrait>;
}
