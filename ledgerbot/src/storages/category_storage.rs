use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;
use yoroolbot::{command_trait::CommandTrait, markdown::MarkdownString, markdown_format};

use crate::commands::{
    command_add_filter::CommandAddFilter, command_categories::CommandCategories,
};

use super::datastore::DataStore;

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

    /// Replace all categories for a specific chat
    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString>;
}

/// Serializable structure for category data that can be saved/loaded as YAML
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Generic category storage that works with any DataStore implementation
/// Maps category name to a list of regex patterns per chat
#[derive(Clone)]
pub struct CategoryStorage<S>
where
    S: DataStore<CategoryData>,
{
    store: S,
    _phantom: PhantomData<CategoryData>,
}

impl<S> CategoryStorage<S>
where
    S: DataStore<CategoryData>,
{
    pub fn new(store: S) -> Self {
        Self {
            store,
            _phantom: PhantomData,
        }
    }

    /// Convert ChatId to string key for datastore
    fn chat_key(chat_id: ChatId) -> String {
        chat_id.0.to_string()
    }
}

/// Implement CategoryStorageTrait for CategoryStorage
#[async_trait::async_trait]
impl<S> CategoryStorageTrait for CategoryStorage<S>
where
    S: DataStore<CategoryData>,
{
    async fn get_chat_categories(
        &self,
        chat_id: ChatId,
    ) -> Result<HashMap<String, Vec<String>>, MarkdownString> {
        let key = Self::chat_key(chat_id);
        Ok(self
            .store
            .get(&key)
            .await
            .map(|data| data.into_hashmap())
            .unwrap_or_default())
    }

    async fn add_category(
        &self,
        chat_id: ChatId,
        category_name: String,
    ) -> Result<(), MarkdownString> {
        let key = Self::chat_key(chat_id);
        let mut data = self
            .store
            .get(&key)
            .await
            .unwrap_or_else(CategoryData::new);

        // Check if category already exists
        if data.categories.contains_key(&category_name) {
            return Err(markdown_format!(
                "ℹ️ Category `{}` already exists\\. Use {} to add more patterns or {} to view all\\.",
                category_name,
                CommandAddFilter::default().to_command_string(false),
                CommandCategories.to_command_string(false)
            ));
        }

        // Add the new category
        data.categories.insert(category_name.clone(), Vec::new());
        self.store.set(&key, data).await;

        Ok(())
    }

    async fn add_category_filter(
        &self,
        chat_id: ChatId,
        category_name: String,
        regex_pattern: String,
    ) -> Result<(), MarkdownString> {
        let key = Self::chat_key(chat_id);
        let mut data = self
            .store
            .get(&key)
            .await
            .unwrap_or_else(CategoryData::new);

        let Some(patterns) = data.categories.get_mut(&category_name) else {
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
        self.store.set(&key, data).await;
        Ok(())
    }

    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) -> Result<(), MarkdownString> {
        let key = Self::chat_key(chat_id);
        let mut data = self
            .store
            .get(&key)
            .await
            .ok_or_else(|| markdown_format!("Category {} not exists", category_name))?;

        let Some(patterns) = data.categories.get_mut(category_name) else {
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
        self.store.set(&key, data).await;
        Ok(())
    }

    async fn remove_category(
        &self,
        chat_id: ChatId,
        category_name: &str,
    ) -> Result<(), MarkdownString> {
        let key = Self::chat_key(chat_id);
        let mut data = self
            .store
            .get(&key)
            .await
            .ok_or_else(|| markdown_format!("Category {} not exists", category_name))?;

        if data.categories.remove(category_name).is_none() {
            return Err(markdown_format!("Category {} not exists", category_name));
        }

        self.store.set(&key, data).await;
        Ok(())
    }

    async fn rename_category(
        &self,
        chat_id: ChatId,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), MarkdownString> {
        let key = Self::chat_key(chat_id);
        let mut data = self
            .store
            .get(&key)
            .await
            .ok_or_else(|| markdown_format!("Category {} not exists", old_name))?;

        if !data.categories.contains_key(old_name) {
            return Err(markdown_format!("Category {} not exists", old_name));
        }
        if data.categories.contains_key(new_name) {
            return Err(markdown_format!("Category {} already exists", new_name));
        }

        let patterns = data.categories.remove(old_name).unwrap();
        data.categories.insert(new_name.to_string(), patterns);
        self.store.set(&key, data).await;
        Ok(())
    }

    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString> {
        let key = Self::chat_key(chat_id);
        let data = CategoryData::from_hashmap(categories);
        self.store.set(&key, data).await;
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
