use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use teloxide::types::ChatId;
use tokio::{fs, sync::Mutex};
use yoroolbot::{command_trait::CommandTrait, markdown::MarkdownString, markdown_format};

use crate::{
    commands::{command_add_filter::CommandAddFilter, command_categories::CommandCategories},
};

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

    /// Rename a category for a specific chat
    async fn rename_category(
        &self,
        chat_id: ChatId,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), MarkdownString>;

    /// Clear all categories for a specific chat
    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString>;
}

type CategoryStorageData = Arc<Mutex<HashMap<ChatId, HashMap<String, Vec<String>>>>>;

/// Serializable structure for category data that can be saved/loaded as YAML
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryData {
    /// Maps category name to a list of regex patterns
    pub categories: HashMap<String, Vec<String>>,
}

impl CategoryData {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn from_hashmap(categories: HashMap<String, Vec<String>>) -> Self {
        Self { categories }
    }

    pub fn into_hashmap(self) -> HashMap<String, Vec<String>> {
        self.categories
    }
}

impl Default for CategoryData {
    fn default() -> Self {
        Self::new()
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
    async fn get_chat_categories(
        &self,
        chat_id: ChatId,
    ) -> Result<HashMap<String, Vec<String>>, MarkdownString> {
        let storage_guard = self.data.lock().await;
        Ok(storage_guard.get(&chat_id).cloned().unwrap_or_default())
    }

    async fn add_category(
        &self,
        chat_id: ChatId,
        category_name: String,
    ) -> Result<(), MarkdownString> {
        // Acquire lock once and hold it for the entire operation to prevent race conditions
        let mut storage_guard = self.data.lock().await;
        let chat_categories = storage_guard.entry(chat_id).or_default();

        // Check if category already exists (while holding the lock)
        if chat_categories.contains_key(&category_name) {
            return Err(markdown_format!(
                "ℹ️ Category `{}` already exists\\. Use {} to add more patterns or {} to view all\\.",
                category_name,
                CommandAddFilter::default().to_command_string(false),
                CommandCategories.to_command_string(false)
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
    ) -> Result<(), MarkdownString> {
        let mut storage_guard = self.data.lock().await;
        let chat_categories = storage_guard.entry(chat_id).or_default();
        let Some(patterns) = chat_categories.get_mut(&category_name) else {
            return Err(markdown_format!("Category {} not exists", category_name));
        };
        if patterns.contains(&regex_pattern) {
            return Err(markdown_format!(
                "Filter `{}` already exists in category `{}`",
                regex_pattern,
                category_name
            ));
        }
        patterns.push(regex_pattern);
        Ok(())
    }

    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) -> Result<(), MarkdownString> {
        let mut storage_guard = self.data.lock().await;
        let Some(chat_categories) = storage_guard.get_mut(&chat_id) else {
            return Err(markdown_format!("Category {} not exists", category_name));
        };
        let Some(patterns) = chat_categories.get_mut(category_name) else {
            return Err(markdown_format!("Category {} not exists", category_name));
        };
        if !patterns.contains(&regex_pattern.to_string()) {
            return Err(markdown_format!(
                "Filter `{}` does not exist in category `{}`",
                regex_pattern,
                category_name
            ));
        }
        patterns.retain(|p| p != regex_pattern);
        Ok(())
    }

    async fn remove_category(
        &self,
        chat_id: ChatId,
        category_name: &str,
    ) -> Result<(), MarkdownString> {
        let mut storage_guard = self.data.lock().await;
        let Some(chat_categories) = storage_guard.get_mut(&chat_id) else {
            return Err(markdown_format!("Category {} not exists", category_name));
        };
        if chat_categories.remove(category_name).is_none() {
            return Err(markdown_format!("Category {} not exists", category_name));
        }
        Ok(())
    }

    async fn rename_category(
        &self,
        chat_id: ChatId,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), MarkdownString> {
        let mut storage_guard = self.data.lock().await;
        let Some(chat_categories) = storage_guard.get_mut(&chat_id) else {
            return Err(markdown_format!("Category {} not exists", old_name));
        };
        if !chat_categories.contains_key(old_name) {
            return Err(markdown_format!("Category {} not exists", old_name));
        }
        if chat_categories.contains_key(new_name) {
            return Err(markdown_format!("Category {} already exists", new_name));
        }
        let patterns = chat_categories.remove(old_name).unwrap();
        chat_categories.insert(new_name.to_string(), patterns);
        Ok(())
    }

    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString> {
        let mut storage_guard = self.data.lock().await;
        storage_guard.insert(chat_id, categories);
        Ok(())
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
        self.storage_dir.join(format!("{}.yaml", chat_id))
    }

    /// Load categories from disk for a specific chat ID
    async fn load_chat_categories(&self, chat_id: ChatId) -> HashMap<String, Vec<String>> {
        let file_path = self.get_file_path(chat_id);

        match fs::read_to_string(&file_path).await {
            Ok(content) => {
                match serde_yaml::from_str::<CategoryData>(&content) {
                    Ok(category_data) => category_data.into_hashmap(),
                    Err(_) => {
                        // Failed to parse YAML, return empty categories
                        HashMap::new()
                    }
                }
            }
            Err(_) => {
                // File doesn't exist or can't be read, return empty categories
                HashMap::new()
            }
        }
    }

    /// Save categories to disk for a specific chat ID
    async fn save_chat_categories(
        &self,
        chat_id: ChatId,
        categories: &HashMap<String, Vec<String>>,
    ) -> Result<(), std::io::Error> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&self.storage_dir).await?;

        let file_path = self.get_file_path(chat_id);
        let category_data = CategoryData::from_hashmap(categories.clone());

        match serde_yaml::to_string(&category_data) {
            Ok(content) => fs::write(&file_path, content).await,
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize categories to YAML: {}", e),
            )),
        }
    }

    /// Ensure categories are loaded for a chat ID (lazy loading)
    async fn ensure_loaded(&self, chat_id: ChatId) -> Result<(), MarkdownString> {
        let loaded_guard = self.loaded_chats.lock().await;
        if loaded_guard.get(&chat_id).copied().unwrap_or(false) {
            // Already loaded
            return Ok(());
        }
        // Not loaded yet, load from disk
        drop(loaded_guard); // Release lock while doing I/O - TODO: what if someone else loads meanwhile?
        let categories = self.load_chat_categories(chat_id).await;
        self.memory_storage
            .replace_categories(chat_id, categories)
            .await
    }
}

/// Implement CategoryStorageTrait for PersistentCategoryStorage
#[async_trait::async_trait]
impl CategoryStorageTrait for PersistentCategoryStorage {
    async fn get_chat_categories(
        &self,
        chat_id: ChatId,
    ) -> Result<HashMap<String, Vec<String>>, MarkdownString> {
        self.ensure_loaded(chat_id).await?;
        self.memory_storage.get_chat_categories(chat_id).await
    }

    async fn add_category(
        &self,
        chat_id: ChatId,
        category_name: String,
    ) -> Result<(), MarkdownString> {
        self.ensure_loaded(chat_id).await?;
        let result = self
            .memory_storage
            .add_category(chat_id, category_name.clone())
            .await;

        if result.is_ok() {
            // Save updated categories to disk
            let categories = self.memory_storage.get_chat_categories(chat_id).await?;
            self.save_chat_categories(chat_id, &categories)
                .await
                .map_err(|e| markdown_format!("{}", e.to_string()))?;
        }

        result
    }

    async fn add_category_filter(
        &self,
        chat_id: ChatId,
        category_name: String,
        regex_pattern: String,
    ) -> Result<(), MarkdownString> {
        self.ensure_loaded(chat_id).await?;
        self.memory_storage
            .add_category_filter(chat_id, category_name, regex_pattern)
            .await?;

        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await?;
        self.save_chat_categories(chat_id, &categories)
            .await
            .map_err(|e| markdown_format!("{}", e.to_string()))?;
        Ok(())
    }

    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) -> Result<(), MarkdownString> {
        self.ensure_loaded(chat_id).await?;
        self.memory_storage
            .remove_category_filter(chat_id, category_name, regex_pattern)
            .await?;

        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await?;
        self.save_chat_categories(chat_id, &categories)
            .await
            .map_err(|e| markdown_format!("{}", e.to_string()))?;
        Ok(())
    }

    async fn remove_category(
        &self,
        chat_id: ChatId,
        category_name: &str,
    ) -> Result<(), MarkdownString> {
        self.ensure_loaded(chat_id).await?;
        self.memory_storage
            .remove_category(chat_id, category_name)
            .await?;

        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await?;
        self.save_chat_categories(chat_id, &categories)
            .await
            .map_err(|e| markdown_format!("{}", e.to_string()))?;
        Ok(())
    }

    async fn rename_category(
        &self,
        chat_id: ChatId,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), MarkdownString> {
        self.ensure_loaded(chat_id).await?;
        self.memory_storage
            .rename_category(chat_id, old_name, new_name)
            .await?;

        // Save updated categories to disk
        let categories = self.memory_storage.get_chat_categories(chat_id).await?;
        self.save_chat_categories(chat_id, &categories)
            .await
            .map_err(|e| markdown_format!("{}", e.to_string()))?;
        Ok(())
    }

    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString> {
        // do not "ensure_loaded" here - we are replacing anyway
        self.memory_storage
            .replace_categories(chat_id, categories)
            .await?;
        let updated_categories = self.memory_storage.get_chat_categories(chat_id).await?;
        self.save_chat_categories(chat_id, &updated_categories)
            .await
            .map_err(|e| markdown_format!("{}", e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_category_data_yaml_serialization() {
        let mut categories = HashMap::new();
        categories.insert(
            "food".to_string(),
            vec!["restaurant".to_string(), "grocery".to_string()],
        );
        categories.insert(
            "transport".to_string(),
            vec!["uber".to_string(), "taxi".to_string(), "bus".to_string()],
        );

        let category_data = CategoryData::from_hashmap(categories.clone());

        // Test serialization to YAML
        let yaml_str = serde_yaml::to_string(&category_data).expect("Failed to serialize to YAML");

        // Verify YAML contains expected content
        assert!(yaml_str.contains("categories:"));
        assert!(yaml_str.contains("food:"));
        assert!(yaml_str.contains("transport:"));
        assert!(yaml_str.contains("- restaurant"));
        assert!(yaml_str.contains("- grocery"));
        assert!(yaml_str.contains("- uber"));

        // Test deserialization from YAML
        let deserialized: CategoryData =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize from YAML");
        let deserialized_map = deserialized.into_hashmap();

        // Verify the deserialized data matches original
        assert_eq!(deserialized_map, categories);
    }

    #[test]
    fn test_category_data_empty() {
        let category_data = CategoryData::new();

        // Test serialization of empty data
        let yaml_str =
            serde_yaml::to_string(&category_data).expect("Failed to serialize empty data");
        assert!(yaml_str.contains("categories: {}"));

        // Test deserialization of empty data
        let deserialized: CategoryData =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize empty data");
        assert!(deserialized.into_hashmap().is_empty());
    }
}
