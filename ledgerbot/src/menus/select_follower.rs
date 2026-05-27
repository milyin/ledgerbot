use std::sync::Arc;

use telluride::{markdown::MarkdownString, markdown_format};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::{Command, command_add_follower::CommandAddFollower},
    menus::common::create_buttons_menu,
    storages::{FollowersStorageTrait, TelegramUsername},
    ui::CommandContext,
};

pub async fn select_follower(
    target: &CommandContext,
    storage: &Arc<dyn FollowersStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&TelegramUsername) -> Command,
    back_command: Option<Command>,
) -> ResponseResult<()> {
    let followers = storage.get_followers().await.unwrap_or_default();
    if followers.is_empty() {
        target
            .send_markdown_message(markdown_format!(
                "👥 No usernames in follower list yet\\. Use {} to add one\\.",
                CommandAddFollower::default().to_command_string(true)
            ))
            .await?;
        return Ok(());
    }

    let msg = target.markdown_message(prompt).await?;
    let menu = create_followers_menu(&followers, next_command, back_command, false);
    let keyboard = target.keyboard(menu).await;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub fn create_followers_menu(
    users: &[TelegramUsername],
    operation: impl Fn(&TelegramUsername) -> Command,
    back_command: Option<Command>,
    inline: bool,
) -> Vec<Vec<crate::ui::ButtonData>> {
    let texts = users
        .iter()
        .map(|username| format!("👤 {}", username.as_str()))
        .collect::<Vec<_>>();
    let values = users.iter().map(operation).collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
