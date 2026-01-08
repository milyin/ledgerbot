use std::sync::Arc;

use telluride::{markdown::MarkdownString, markdown_format};
use teloxide::types::ChatId;
use yoroolbot::storage::DataStoreTrait;

use crate::storages::TelegramUsername;

/// Trait for followers storage read operations
#[async_trait::async_trait]
pub trait FollowersStorageReadTrait: Send + Sync {
    /// Get all follower usernames
    async fn get_followers(&self) -> Result<Vec<TelegramUsername>, MarkdownString>;
}

/// Trait for followers storage operations
#[async_trait::async_trait]
pub trait FollowersStorageTrait: FollowersStorageReadTrait + Send + Sync {
    /// Add a username to the follower list
    async fn add_follower(&self, username: &TelegramUsername) -> Result<(), MarkdownString>;

    /// Remove a username from the follower list
    async fn remove_follower(&self, username: &TelegramUsername) -> Result<(), MarkdownString>;

    /// Replace all followers
    async fn replace_followers(&self, users: Vec<TelegramUsername>) -> Result<(), MarkdownString>;
}

/// Type alias for followers data (list of username strings)
pub type FollowersData = Vec<String>;

/// Generic followers storage that works with any DataStore implementation
/// Stores followers as a single key "followers" with a list of usernames as the value
#[derive(Clone)]
pub struct FollowersStorage {
    store: Arc<dyn DataStoreTrait<FollowersData>>,
    chat_id: ChatId,
}

impl FollowersStorage {
    /// Create a new FollowersStorage with the given DataStore and chat ID
    pub fn new(store: Arc<dyn DataStoreTrait<FollowersData>>, chat_id: ChatId) -> Self {
        Self { store, chat_id }
    }
}

const FOLLOWERS_KEY: &str = "followers";

/// Implement FollowersStorageReadTrait for FollowersStorage
#[async_trait::async_trait]
impl FollowersStorageReadTrait for FollowersStorage {
    async fn get_followers(&self) -> Result<Vec<TelegramUsername>, MarkdownString> {
        // Get all usernames stored under the key
        let username_strings = self
            .store
            .get(self.chat_id, FOLLOWERS_KEY)
            .await
            .unwrap_or_default();

        // Parse each string into TelegramUsername
        let mut followers = Vec::new();
        for username_str in username_strings {
            match TelegramUsername::from_string(&username_str) {
                Ok(username) => followers.push(username),
                Err(e) => {
                    log::warn!(
                        "Invalid follower username in storage: {}: {}",
                        username_str,
                        e
                    );
                    // Skip invalid usernames
                }
            }
        }

        Ok(followers)
    }
}

/// Implement FollowersStorageTrait for FollowersStorage
#[async_trait::async_trait]
impl FollowersStorageTrait for FollowersStorage {
    async fn add_follower(&self, username: &TelegramUsername) -> Result<(), MarkdownString> {
        // Get existing followers
        let mut follower_strings = self
            .store
            .get(self.chat_id, FOLLOWERS_KEY)
            .await
            .unwrap_or_default();

        // Check if already exists
        let username_str = username.as_str().to_string();
        if follower_strings.contains(&username_str) {
            return Err(markdown_format!(
                "ℹ️ Username `{}` is already in the follower list\\.",
                username.as_str()
            ));
        }

        // Add new username
        follower_strings.push(username_str);
        self.store
            .set(self.chat_id, FOLLOWERS_KEY, follower_strings)
            .await;

        Ok(())
    }

    async fn remove_follower(&self, username: &TelegramUsername) -> Result<(), MarkdownString> {
        // Get existing followers
        let mut follower_strings = self
            .store
            .get(self.chat_id, FOLLOWERS_KEY)
            .await
            .unwrap_or_default();

        let username_str = username.as_str();

        // Check if exists
        if !follower_strings.contains(&username_str.to_string()) {
            return Err(markdown_format!(
                "❌ Username `{}` is not in the follower list\\.",
                username_str
            ));
        }

        // Remove username
        follower_strings.retain(|s| s != username_str);
        self.store
            .set(self.chat_id, FOLLOWERS_KEY, follower_strings)
            .await;

        Ok(())
    }

    async fn replace_followers(&self, users: Vec<TelegramUsername>) -> Result<(), MarkdownString> {
        // Convert to strings
        let user_strings: Vec<String> = users.iter().map(|u| u.as_str().to_string()).collect();

        // Replace all followers
        self.store
            .set(self.chat_id, FOLLOWERS_KEY, user_strings)
            .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use yoroolbot::storage::InMemStore;

    use super::*;

    #[tokio::test]
    async fn test_add_and_get_followers() {
        let chat_id = ChatId(123);
        let storage = FollowersStorage::new(Arc::new(InMemStore::<FollowersData>::new()), chat_id);

        let user1 = TelegramUsername::from_string("@alice").unwrap();
        let user2 = TelegramUsername::from_string("@bobby").unwrap();

        // Add first follower
        storage.add_follower(&user1).await.unwrap();

        // Get followers
        let followers = storage.get_followers().await.unwrap();
        assert_eq!(followers.len(), 1);
        assert_eq!(followers[0].as_str(), "@alice");

        // Add second follower
        storage.add_follower(&user2).await.unwrap();

        // Get followers again
        let followers = storage.get_followers().await.unwrap();
        assert_eq!(followers.len(), 2);
    }

    #[tokio::test]
    async fn test_add_duplicate_follower() {
        let chat_id = ChatId(123);
        let storage = FollowersStorage::new(Arc::new(InMemStore::<FollowersData>::new()), chat_id);

        let user1 = TelegramUsername::from_string("@alice").unwrap();

        // Add first time - should succeed
        storage.add_follower(&user1).await.unwrap();

        // Add again - should fail
        let result = storage.add_follower(&user1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_follower() {
        let chat_id = ChatId(123);
        let storage = FollowersStorage::new(Arc::new(InMemStore::<FollowersData>::new()), chat_id);

        let user1 = TelegramUsername::from_string("@alice").unwrap();

        // Add follower
        storage.add_follower(&user1).await.unwrap();

        // Remove follower
        storage.remove_follower(&user1).await.unwrap();

        // Get followers - should be empty
        let followers = storage.get_followers().await.unwrap();
        assert_eq!(followers.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_follower() {
        let chat_id = ChatId(123);
        let storage = FollowersStorage::new(Arc::new(InMemStore::<FollowersData>::new()), chat_id);

        let user1 = TelegramUsername::from_string("@alice").unwrap();

        // Try to remove without adding - should fail
        let result = storage.remove_follower(&user1).await;
        assert!(result.is_err());
    }
}
