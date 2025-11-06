use std::sync::Arc;

use teloxide::types::ChatId;
use yoroolbot::{markdown::MarkdownString, markdown_format, storage::DataStoreTrait};

use crate::storages::ShareUsername;


/// Trait for share storage read operations
#[async_trait::async_trait]
pub trait ShareStorageReadTrait: Send + Sync {
    /// Get all shared usernames
    async fn get_shares(&self) -> Result<Vec<ShareUsername>, MarkdownString>;
}

/// Trait for share storage operations
#[async_trait::async_trait]
pub trait ShareStorageTrait: ShareStorageReadTrait + Send + Sync {
    /// Add a username to the share list
    async fn add_share(
        &self,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString>;

    /// Remove a username from the share list
    async fn remove_share(
        &self,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString>;

    /// Replace all shares
    async fn replace_shares(
        &self,
        shares: Vec<ShareUsername>,
    ) -> Result<(), MarkdownString>;
}

/// Type alias for share data (list of username strings)
pub type ShareData = Vec<String>;

/// Generic share storage that works with any DataStore implementation
/// Stores shares as a single key "shares" with a list of usernames as the value
#[derive(Clone)]
pub struct ShareStorage
{
    store: Arc<dyn DataStoreTrait<ShareData>>,
    chat_id: ChatId,
}

impl ShareStorage
{
    /// Create a new ShareStorage with the given DataStore and chat ID
    pub fn new(store: Arc<dyn DataStoreTrait<ShareData>>, chat_id: ChatId) -> Self {
        Self { store, chat_id }
    }
}

/// Implement ShareStorageReadTrait for ShareStorage
#[async_trait::async_trait]
impl ShareStorageReadTrait for ShareStorage
{
    async fn get_shares(&self) -> Result<Vec<ShareUsername>, MarkdownString> {
        // Get all usernames stored under the "shares" key
        let share_strings = self.store.get(self.chat_id, "shares").await.unwrap_or_default();

        // Parse each string into ShareUsername
        let mut shares = Vec::new();
        for username_str in share_strings {
            match ShareUsername::from_string(&username_str) {
                Ok(username) => shares.push(username),
                Err(e) => {
                    log::warn!("Invalid share username in storage: {}: {}", username_str, e);
                    // Skip invalid usernames
                }
            }
        }

        Ok(shares)
    }
}

/// Implement ShareStorageTrait for ShareStorage
#[async_trait::async_trait]
impl ShareStorageTrait for ShareStorage
{
    async fn add_share(
        &self,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString> {
        // Get existing shares
        let mut share_strings = self.store.get(self.chat_id, "shares").await.unwrap_or_default();

        // Check if already exists
        let username_str = username.as_str().to_string();
        if share_strings.contains(&username_str) {
            return Err(markdown_format!(
                "ℹ️ Username `{}` is already in the share list\\.",
                username.as_str()
            ));
        }

        // Add new username
        share_strings.push(username_str);
        self.store.set(self.chat_id, "shares", share_strings).await;

        Ok(())
    }

    async fn remove_share(
        &self,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString> {
        // Get existing shares
        let mut share_strings = self.store.get(self.chat_id, "shares").await.unwrap_or_default();

        let username_str = username.as_str();

        // Check if exists
        if !share_strings.contains(&username_str.to_string()) {
            return Err(markdown_format!(
                "❌ Username `{}` is not in the share list\\.",
                username_str
            ));
        }

        // Remove username
        share_strings.retain(|s| s != username_str);
        self.store.set(self.chat_id, "shares", share_strings).await;

        Ok(())
    }

    async fn replace_shares(
        &self,
        shares: Vec<ShareUsername>,
    ) -> Result<(), MarkdownString> {
        // Convert to strings
        let share_strings: Vec<String> = shares.iter().map(|u| u.as_str().to_string()).collect();

        // Replace all shares
        self.store.set(self.chat_id, "shares", share_strings).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use yoroolbot::storage::InMemStore;

    use super::*;

    #[tokio::test]
    async fn test_add_and_get_shares() {
        let chat_id = ChatId(123);
        let storage = ShareStorage::new(Arc::new(InMemStore::<ShareData>::new()), chat_id);

        let user1 = ShareUsername::from_string("@alice").unwrap();
        let user2 = ShareUsername::from_string("@bobby").unwrap();

        // Add first share
        storage.add_share(&user1).await.unwrap();

        // Get shares
        let shares = storage.get_shares().await.unwrap();
        assert_eq!(shares.len(), 1);
        assert_eq!(shares[0].as_str(), "@alice");

        // Add second share
        storage.add_share(&user2).await.unwrap();

        // Get shares again
        let shares = storage.get_shares().await.unwrap();
        assert_eq!(shares.len(), 2);
    }

    #[tokio::test]
    async fn test_add_duplicate_share() {
        let chat_id = ChatId(123);
        let storage = ShareStorage::new(Arc::new(InMemStore::<ShareData>::new()), chat_id);

        let user1 = ShareUsername::from_string("@alice").unwrap();

        // Add first time - should succeed
        storage.add_share(&user1).await.unwrap();

        // Add again - should fail
        let result = storage.add_share(&user1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_share() {
        let chat_id = ChatId(123);
        let storage = ShareStorage::new(Arc::new(InMemStore::<ShareData>::new()), chat_id);

        let user1 = ShareUsername::from_string("@alice").unwrap();

        // Add share
        storage.add_share(&user1).await.unwrap();

        // Remove share
        storage.remove_share(&user1).await.unwrap();

        // Get shares - should be empty
        let shares = storage.get_shares().await.unwrap();
        assert_eq!(shares.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_share() {
        let chat_id = ChatId(123);
        let storage = ShareStorage::new(Arc::new(InMemStore::<ShareData>::new()), chat_id);

        let user1 = ShareUsername::from_string("@alice").unwrap();

        // Try to remove without adding - should fail
        let result = storage.remove_share(&user1).await;
        assert!(result.is_err());
    }
}
