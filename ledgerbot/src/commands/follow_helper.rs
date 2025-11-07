use std::sync::Arc;

use teloxide::types::ChatId;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    markdown_format,
};

use crate::{
    commands::command_add_follower::CommandAddFollower,
    storages::{StorageReadonly, Stores},
};

/// Validates if the current user has access to follow the stored follow chat
/// Returns the effective chat ID to use and an optional header note
///
/// This function:
/// 1. Checks if a follow chat is set in VariableStorage
/// 2. If set, validates the current user has access by checking the followers list
/// 3. Returns the effective chat ID (followed or current) and a header note
pub async fn validate_and_get_follow_access(
    target: &CommandReplyTarget,
    stores: &Arc<Stores>,
) -> Result<Arc<StorageReadonly>, MarkdownString> {
    let storage = stores.storage(target.chat.id);
    let variable_storage = storage.variables();

    // Check if currently following any chat
    let followed_chat: Option<ChatId> = variable_storage.get().await;
    let followed_chat = match followed_chat {
        Some(chat_id) => chat_id,
        None => {
            // Not following anyone, use current chat
            return Ok(stores.storage_readonly(target.chat.id, target.chat.id));
        }
    };

    // Get current user's username
    let current_username = match &target.chat.username() {
        Some(username) => format!("@{}", username),
        None => {
            // User doesn't have username anymore, can't validate access
            // Clear the follow and use current chat
            variable_storage.remove::<Option<ChatId>>().await;

            return Err(markdown_format!(
                "⚠️ You don't have a Telegram username\\. Follow access has been cleared\\. Using current chat data\\."
            ));
        }
    };

    // Validate access by checking followers list
    let external_storage = stores.storage_readonly(target.chat.id, followed_chat);
    let followers_storage = external_storage.followers_readonly();
    let followers = match followers_storage.get_followers().await {
        Ok(followers) => followers,
        Err(_) => {
            // Can't access followers list, clear follow and use current chat
            variable_storage.remove::<Option<ChatId>>().await;

            return Err(markdown_format!(
                "⚠️ Cannot access followers list from chat `{}`\\. Follow access has been cleared\\. Using current chat data\\.",
                followed_chat.0
            ));
        }
    };

    // Check if current user is still in the followers list
    let user_in_list = followers
        .iter()
        .any(|username| username.as_str() == current_username);

    if !user_in_list {
        // User no longer has access, clear follow and use current chat
        variable_storage.remove::<Option<ChatId>>().await;

        return Err(markdown_format!(
            "⚠️ You no longer have access to chat `{}`\\. Follow access has been cleared\\. Using current chat data\\.",
            followed_chat.0
        ));
    }

    // Access validated, return external storage
    Ok(stores.storage_readonly(target.chat.id, followed_chat))
}

/// Validates if the current user has access to follow a specific target chat
/// This is used by the /follow command when setting up a new follow relationship
pub async fn validate_follow_access(
    target: &CommandReplyTarget,
    storage: &Arc<Stores>,
    target_chat_id: ChatId,
) -> Result<(), MarkdownString> {
    // Get current user's username
    let current_username = match &target.chat.username() {
        Some(username) => format!("@{}", username),
        None => {
            return Err(markdown_format!(
                "❌ You need to have a Telegram username to use this feature\\. Please set a username in your Telegram settings\\."
            ));
        }
    };

    // Get followers list from target chat
    let storage = storage.storage(target_chat_id);
    let followers_storage = storage.followers();
    let users = match followers_storage.get_followers().await {
        Ok(users) => users,
        Err(e) => {
            return Err(markdown_format!(
                "❌ Failed to get followers list from chat `{}`\\: {}",
                target_chat_id.0,
                e.to_string()
            ));
        }
    };

    // Check if current user is in the followers list
    let user_in_list = users
        .iter()
        .any(|username| username.as_str() == current_username);

    if !user_in_list {
        return Err(markdown_format!(
            "❌ You are not in the followers list for chat `{}`\\. Ask the chat owner to add you using {}",
            target_chat_id.0,
            CommandAddFollower::new(current_username).to_command_string(true)
        ));
    }

    Ok(())
}
