use std::sync::Arc;

use telluride::{markdown::MarkdownString, markdown_format};
use teloxide::types::ChatId;

use crate::{
    commands::command_add_follower::CommandAddFollower,
    storages::{StorageReadonly, Stores},
    ui::CommandContext,
};

pub async fn validate_and_get_follow_access(
    target: &CommandContext,
    stores: &Arc<Stores>,
) -> Result<Arc<StorageReadonly>, MarkdownString> {
    let storage = stores.storage(target.chat.id);
    let variable_storage = storage.variables();

    let followed_chat: Option<ChatId> = variable_storage.get().await;
    let followed_chat = match followed_chat {
        Some(chat_id) => chat_id,
        None => return Ok(stores.storage_readonly(target.chat.id, target.chat.id)),
    };

    let current_username = match &target.chat.username() {
        Some(username) => format!("@{}", username),
        None => {
            variable_storage.remove::<Option<ChatId>>().await;
            return Err(markdown_format!(
                "⚠️ You don't have a Telegram username\\. Follow access has been cleared\\. Using current chat data\\."
            ));
        }
    };

    let external_storage = stores.storage_readonly(target.chat.id, followed_chat);
    let followers_storage = external_storage.followers_readonly();
    let followers = match followers_storage.get_followers().await {
        Ok(followers) => followers,
        Err(_) => {
            variable_storage.remove::<Option<ChatId>>().await;
            return Err(markdown_format!(
                "⚠️ Cannot access followers list from chat `{}`\\. Follow access has been cleared\\. Using current chat data\\.",
                followed_chat.0
            ));
        }
    };

    let user_in_list = followers
        .iter()
        .any(|username| username.as_str() == current_username);

    if !user_in_list {
        variable_storage.remove::<Option<ChatId>>().await;
        return Err(markdown_format!(
            "⚠️ You no longer have access to chat `{}`\\. Follow access has been cleared\\. Using current chat data\\.",
            followed_chat.0
        ));
    }

    Ok(stores.storage_readonly(target.chat.id, followed_chat))
}

pub async fn validate_follow_access(
    target: &CommandContext,
    storage: &Arc<Stores>,
    target_chat_id: ChatId,
) -> Result<(), MarkdownString> {
    let current_username = match &target.chat.username() {
        Some(username) => format!("@{}", username),
        None => {
            return Err(markdown_format!(
                "❌ You need to have a Telegram username to use this feature\\. Please set a username in your Telegram settings\\."
            ));
        }
    };

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
