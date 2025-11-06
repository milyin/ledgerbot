use std::sync::Arc;

use teloxide::types::ChatId;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    markdown_format,
};

use crate::{commands::command_add_share::CommandAddShare, storages::Storage};

/// Result of follow access validation
pub struct FollowAccess {
    /// The chat ID to use for accessing data (either current chat or followed chat)
    pub effective_chat_id: ChatId,
    /// Optional header message to display in reports when following another chat
    pub header_note: Option<MarkdownString>,
}

/// Validates if the current user has access to follow the stored follow chat
/// Returns the effective chat ID to use and an optional header note
///
/// This function:
/// 1. Checks if a follow chat is set in VariableStorage
/// 2. If set, validates the current user has access by checking the share list
/// 3. Returns the effective chat ID (followed or current) and a header note
pub async fn validate_and_get_follow_access(
    target: &CommandReplyTarget,
    storage: &Arc<Storage>,
) -> Result<FollowAccess, MarkdownString> {
    let variable_storage = storage.clone().variable_storage();

    // Check if currently following any chat
    let followed_chat: Option<ChatId> = variable_storage.get(target.chat.id).await;

    let followed_chat = match followed_chat {
        Some(chat_id) => chat_id,
        None => {
            // Not following anyone, use current chat
            return Ok(FollowAccess {
                effective_chat_id: target.chat.id,
                header_note: None,
            });
        }
    };

    // Get current user's username
    let current_username = match &target.chat.username() {
        Some(username) => format!("@{}", username),
        None => {
            // User doesn't have username anymore, can't validate access
            // Clear the follow and use current chat
            variable_storage
                .remove::<Option<ChatId>>(target.chat.id)
                .await;

            return Err(markdown_format!(
                "⚠️ You no longer have a Telegram username\\. Follow access has been cleared\\. Using current chat data\\."
            ));
        }
    };

    // Validate access by checking share list
    let share_storage = storage.clone().share_storage();
    let shares = match share_storage.get_chat_shares(followed_chat).await {
        Ok(shares) => shares,
        Err(_) => {
            // Can't access share list, clear follow and use current chat
            variable_storage
                .remove::<Option<ChatId>>(target.chat.id)
                .await;

            return Err(markdown_format!(
                "⚠️ Cannot access share list from chat `{}`\\. Follow access has been cleared\\. Using current chat data\\.",
                followed_chat.0
            ));
        }
    };

    // Check if current user is still in the share list
    let user_in_list = shares
        .iter()
        .any(|share_username| share_username.as_str() == current_username);

    if !user_in_list {
        // User no longer has access, clear follow and use current chat
        variable_storage
            .remove::<Option<ChatId>>(target.chat.id)
            .await;

        return Err(markdown_format!(
            "⚠️ You no longer have access to chat `{}`\\. Follow access has been cleared\\. Using current chat data\\.",
            followed_chat.0
        ));
    }

    // Access validated, return followed chat with header note
    Ok(FollowAccess {
        effective_chat_id: followed_chat,
        header_note: Some(markdown_format!(
            "👁️ **Following expenses from chat `{}`**\n\n",
            followed_chat.0
        )),
    })
}

/// Validates if the current user has access to follow a specific target chat
/// This is used by the /follow command when setting up a new follow relationship
pub async fn validate_follow_access(
    target: &CommandReplyTarget,
    storage: &Arc<Storage>,
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

    // Get share list from target chat
    let share_storage = storage.clone().share_storage();
    let shares = match share_storage.get_chat_shares(target_chat_id).await {
        Ok(shares) => shares,
        Err(e) => {
            return Err(markdown_format!(
                "❌ Failed to get share list from chat `{}`\\: {}",
                target_chat_id.0,
                e.to_string()
            ));
        }
    };

    // Check if current user is in the share list
    let user_in_list = shares
        .iter()
        .any(|share_username| share_username.as_str() == current_username);

    if !user_in_list {
        return Err(markdown_format!(
            "❌ You are not in the share list for chat `{}`\\. Ask the chat owner to add you using {}",
            target_chat_id.0,
            CommandAddShare::new(current_username).to_command_string(true)
        ));
    }

    Ok(())
}
