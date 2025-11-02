use std::{collections::HashMap, marker::PhantomData};

use teloxide::types::ChatId;
use yoroolbot::{
    command_trait::CommandTrait, markdown::MarkdownString, markdown_format, storage::DataStore,
};

use crate::commands::{
    command_add_filter::CommandAddFilter, command_categories::CommandCategories,
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

    /// Replace all categories for a specific chat
    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString>;
}

/// Type alias for category filter patterns (list of regex strings)
pub type CategoryData = Vec<String>;

/// Generic category storage that works with any DataStore implementation
/// Each category is stored as a separate key (category name) with its filters as the value
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
        // Get all keys (category names) for this chat
        let category_names = self.store.keys(chat_id).await;
        let mut categories = HashMap::new();

        for category_name in category_names {
            if let Some(filters) = self.store.get(chat_id, &category_name).await {
                categories.insert(category_name, filters);
            }
        }

        Ok(categories)
    }

    async fn add_category(
        &self,
        chat_id: ChatId,
        category_name: String,
    ) -> Result<(), MarkdownString> {
        // Check if category already exists
        if self.store.get(chat_id, &category_name).await.is_some() {
            return Err(markdown_format!(
                "ℹ️ Category `{}` already exists\\. Use {} to add more patterns or {} to view all\\.",
                category_name,
                CommandAddFilter::default().to_command_string(false),
                CommandCategories.to_command_string(false)
            ));
        }

        // Add the new category with empty filters list
        self.store.set(chat_id, &category_name, Vec::new()).await;

        Ok(())
    }

    async fn add_category_filter(
        &self,
        chat_id: ChatId,
        category_name: String,
        regex_pattern: String,
    ) -> Result<(), MarkdownString> {
        // Get existing filters for this category
        let mut patterns = self
            .store
            .get(chat_id, &category_name)
            .await
            .ok_or_else(|| markdown_format!("Category {} not exists", &category_name))?;

        if patterns.contains(&regex_pattern) {
            return Err(markdown_format!(
                "Filter `{}` already exists in category `{}`",
                regex_pattern,
                category_name
            ));
        }

        patterns.push(regex_pattern);
        self.store.set(chat_id, &category_name, patterns).await;
        Ok(())
    }

    async fn remove_category_filter(
        &self,
        chat_id: ChatId,
        category_name: &str,
        regex_pattern: &str,
    ) -> Result<(), MarkdownString> {
        // Get existing filters for this category
        let mut patterns = self
            .store
            .get(chat_id, category_name)
            .await
            .ok_or_else(|| markdown_format!("Category {} not exists", category_name))?;

        if !patterns.contains(&regex_pattern.to_string()) {
            return Err(markdown_format!(
                "Filter `{}` does not exist in category `{}`",
                regex_pattern,
                category_name
            ));
        }

        patterns.retain(|p| p != regex_pattern);
        self.store.set(chat_id, category_name, patterns).await;
        Ok(())
    }

    async fn remove_category(
        &self,
        chat_id: ChatId,
        category_name: &str,
    ) -> Result<(), MarkdownString> {
        // Check if category exists
        if self.store.get(chat_id, category_name).await.is_none() {
            return Err(markdown_format!("Category {} not exists", category_name));
        }

        self.store.remove(chat_id, category_name).await;
        Ok(())
    }

    async fn rename_category(
        &self,
        chat_id: ChatId,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), MarkdownString> {
        // Get existing filters for old category
        let patterns = self
            .store
            .get(chat_id, old_name)
            .await
            .ok_or_else(|| markdown_format!("Category {} not exists", old_name))?;

        // Check if new name already exists
        if self.store.get(chat_id, new_name).await.is_some() {
            return Err(markdown_format!("Category {} already exists", new_name));
        }

        // Create new category with same patterns
        self.store.set(chat_id, new_name, patterns).await;
        // Remove old category
        self.store.remove(chat_id, old_name).await;
        Ok(())
    }

    async fn replace_categories(
        &self,
        chat_id: ChatId,
        categories: HashMap<String, Vec<String>>,
    ) -> Result<(), MarkdownString> {
        // Remove all existing categories for this chat
        let existing_categories = self.store.keys(chat_id).await;
        for category_name in existing_categories {
            self.store.remove(chat_id, &category_name).await;
        }

        // Add all new categories
        for (category_name, filters) in categories {
            self.store.set(chat_id, &category_name, filters).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_data_yaml_serialization() {
        let filters = vec!["restaurant".to_string(), "grocery".to_string()];

        // Test serialization to YAML
        let yaml_str = serde_yaml::to_string(&filters).expect("Failed to serialize to YAML");

        // Verify YAML contains expected content
        assert!(yaml_str.contains("- restaurant"));
        assert!(yaml_str.contains("- grocery"));

        // Test deserialization from YAML
        let deserialized: CategoryData =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize from YAML");

        // Verify the deserialized data matches original
        assert_eq!(deserialized, filters);
    }

    #[test]
    fn test_category_data_empty() {
        let category_data: CategoryData = Vec::new();

        // Test serialization of empty data
        let yaml_str =
            serde_yaml::to_string(&category_data).expect("Failed to serialize empty data");
        assert!(yaml_str.contains("[]"));

        // Test deserialization of empty data
        let deserialized: CategoryData =
            serde_yaml::from_str(&yaml_str).expect("Failed to deserialize empty data");
        assert!(deserialized.is_empty());
    }
}
