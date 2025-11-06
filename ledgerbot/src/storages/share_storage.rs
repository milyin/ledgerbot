use std::marker::PhantomData;

use teloxide::types::ChatId;
use yoroolbot::{markdown::MarkdownString, markdown_format, storage::DataStoreTrait};

use crate::storages::ShareUsername;

/// Trait for share storage operations
#[async_trait::async_trait]
pub trait ShareStorageTrait: Send + Sync {
    /// Get all shared usernames for a specific chat
    async fn get_chat_shares(&self, chat_id: ChatId) -> Result<Vec<ShareUsername>, MarkdownString>;

    /// Add a username to the share list for a specific chat
    async fn add_share(
        &self,
        chat_id: ChatId,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString>;

    /// Remove a username from the share list for a specific chat
    async fn remove_share(
        &self,
        chat_id: ChatId,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString>;

    /// Replace all shares for a specific chat
    async fn replace_shares(
        &self,
        chat_id: ChatId,
        shares: Vec<ShareUsername>,
    ) -> Result<(), MarkdownString>;
}

/// Type alias for share data (list of username strings)
pub type ShareData = Vec<String>;

/// Generic share storage that works with any DataStore implementation
/// Stores shares as a single key "shares" with a list of usernames as the value
#[derive(Clone)]
pub struct ShareStorage<S>
where
    S: DataStoreTrait<ShareData>,
{
    store: S,
    _phantom: PhantomData<ShareData>,
}

impl<S> ShareStorage<S>
where
    S: DataStoreTrait<ShareData>,
{
    pub fn new(store: S) -> Self {
        Self {
            store,
            _phantom: PhantomData,
        }
    }
}

/// Implement ShareStorageTrait for ShareStorage
#[async_trait::async_trait]
impl<S> ShareStorageTrait for ShareStorage<S>
where
    S: DataStoreTrait<ShareData>,
{
    async fn get_chat_shares(&self, chat_id: ChatId) -> Result<Vec<ShareUsername>, MarkdownString> {
        // Get all usernames stored under the "shares" key
        let share_strings = self.store.get(chat_id, "shares").await.unwrap_or_default();

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

    async fn add_share(
        &self,
        chat_id: ChatId,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString> {
        // Get existing shares
        let mut share_strings = self.store.get(chat_id, "shares").await.unwrap_or_default();

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
        self.store.set(chat_id, "shares", share_strings).await;

        Ok(())
    }

    async fn remove_share(
        &self,
        chat_id: ChatId,
        username: &ShareUsername,
    ) -> Result<(), MarkdownString> {
        // Get existing shares
        let mut share_strings = self.store.get(chat_id, "shares").await.unwrap_or_default();

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
        self.store.set(chat_id, "shares", share_strings).await;

        Ok(())
    }

    async fn replace_shares(
        &self,
        chat_id: ChatId,
        shares: Vec<ShareUsername>,
    ) -> Result<(), MarkdownString> {
        // Convert to strings
        let share_strings: Vec<String> = shares.iter().map(|u| u.as_str().to_string()).collect();

        // Replace all shares
        self.store.set(chat_id, "shares", share_strings).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yoroolbot::storage::InMemStore;

    #[tokio::test]
    async fn test_add_and_get_shares() {
        let storage = ShareStorage::new(InMemStore::<ShareData>::new());
        let chat_id = ChatId(123);

        let user1 = ShareUsername::from_string("@alice").unwrap();
        let user2 = ShareUsername::from_string("@bobby").unwrap();

        // Add first share
        storage.add_share(chat_id, &user1).await.unwrap();

        // Get shares
        let shares = storage.get_chat_shares(chat_id).await.unwrap();
        assert_eq!(shares.len(), 1);
        assert_eq!(shares[0].as_str(), "@alice");

        // Add second share
        storage.add_share(chat_id, &user2).await.unwrap();

        // Get shares again
        let shares = storage.get_chat_shares(chat_id).await.unwrap();
        assert_eq!(shares.len(), 2);
    }

    #[tokio::test]
    async fn test_add_duplicate_share() {
        let storage = ShareStorage::new(InMemStore::<ShareData>::new());
        let chat_id = ChatId(123);

        let user1 = ShareUsername::from_string("@alice").unwrap();

        // Add first time - should succeed
        storage.add_share(chat_id, &user1).await.unwrap();

        // Add again - should fail
        let result = storage.add_share(chat_id, &user1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_share() {
        let storage = ShareStorage::new(InMemStore::<ShareData>::new());
        let chat_id = ChatId(123);

        let user1 = ShareUsername::from_string("@alice").unwrap();

        // Add share
        storage.add_share(chat_id, &user1).await.unwrap();

        // Remove share
        storage.remove_share(chat_id, &user1).await.unwrap();

        // Get shares - should be empty
        let shares = storage.get_chat_shares(chat_id).await.unwrap();
        assert_eq!(shares.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_share() {
        let storage = ShareStorage::new(InMemStore::<ShareData>::new());
        let chat_id = ChatId(123);

        let user1 = ShareUsername::from_string("@alice").unwrap();

        // Try to remove without adding - should fail
        let result = storage.remove_share(chat_id, &user1).await;
        assert!(result.is_err());
    }
}
