use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use yoroolbot::markdown::MarkdownString;

// Forward declaration - full import would create circular dependency
use crate::commands::Command;

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
    async fn get_chat_categories(
        &self,
        chat_id: ChatId,
    ) -> Result<HashMap<String, Vec<String>>, MarkdownString>;

    /// Add a category for a specific chat
    async fn add_category(
        &self,
        chat_id: ChatId,
        category_name: String,
    ) -> Result<(), MarkdownString>;

    /// Add a regex filter to an existing category
    async fn add_category_filter(
        &self,
        chat_id: ChatId,
        category_name: String,
        regex_pattern: String,
    ) -> Result<(), MarkdownString>;

    /// Remove a regex filter from a category
    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) -> Result<(), MarkdownString>;

    /// Remove a category from a specific chat
    async fn remove_category(
        &self,
        chat_id: ChatId,
        category_name: &str,
    ) -> Result<(), MarkdownString>;

    /// Clear all categories for a specific chat
    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString>;
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

/// Trait for batch storage operations (temporary command batching)
#[async_trait::async_trait]
pub trait BatchStorageTrait: Send + Sync {
    /// Add commands to batch and return whether this is the first message in the batch
    async fn add_to_batch(&self, chat_id: ChatId, commands: Vec<Result<Command, String>>) -> bool;

    /// Consume and remove batch data for a chat
    async fn consume_batch(&self, chat_id: ChatId) -> Option<Vec<Result<Command, String>>>;
}

/// Trait for callback data storage operations (maps short references to full callback data)
/// This is used to work around Telegram's 64-byte limit on callback data
#[async_trait::async_trait]
pub trait CallbackDataStorageTrait: Send + Sync {
    /// Store callback data and return a short reference string
    /// The reference is based on (message_id, button_position)
    async fn store_callback_data(
        &self,
        chat_id: ChatId,
        message_id: i32,
        button_pos: usize,
        data: String,
    ) -> String;

    /// Retrieve original callback data from a reference string
    async fn get_callback_data(&self, reference: &str) -> Option<String>;

    /// Clear all callback data for a specific message
    async fn clear_message_callbacks(&self, chat_id: ChatId, message_id: i32);
}

/// Combined storage trait that provides all storage operations
/// This trait allows converting to specific trait objects for functions that only need subset of functionality
pub trait StorageTrait: Send + Sync {
    /// Convert to ExpenseStorageTrait trait object
    fn as_expense_storage(self: Arc<Self>) -> Arc<dyn ExpenseStorageTrait>;

    /// Convert to CategoryStorageTrait trait object
    fn as_category_storage(self: Arc<Self>) -> Arc<dyn CategoryStorageTrait>;

    /// Convert to FilterSelectionStorageTrait trait object
    fn as_filter_selection_storage(self: Arc<Self>) -> Arc<dyn FilterSelectionStorageTrait>;

    /// Convert to FilterPageStorageTrait trait object
    fn as_filter_page_storage(self: Arc<Self>) -> Arc<dyn FilterPageStorageTrait>;

    /// Convert to BatchStorageTrait trait object
    fn as_batch_storage(self: Arc<Self>) -> Arc<dyn BatchStorageTrait>;

    /// Convert to CallbackDataStorageTrait trait object
    fn as_callback_data_storage(self: Arc<Self>) -> Arc<dyn CallbackDataStorageTrait>;
}
