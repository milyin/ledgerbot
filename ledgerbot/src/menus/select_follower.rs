use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::InlineKeyboardMarkup,
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    markdown_format,
};

use crate::{
    commands::command_add_follower::CommandAddFollower,
    menus::common::create_buttons_menu,
    storages::{FollowersStorageTrait, TelegramUsername},
};

pub async fn select_follower<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn FollowersStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&TelegramUsername) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let followers = storage
        .get_followers()
        .await
        .unwrap_or_default();
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
    let menu = create_followers_menu(
        &followers,
        |username| next_command(username).to_command_string(false),
        back_command,
        false,
    );
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(menu)
        .await?;
    Ok(())
}

pub fn create_followers_menu(
    users: &[TelegramUsername],
    operation: impl Fn(&TelegramUsername) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = users
        .iter()
        .map(|username| format!("👤 {}", username.as_str()))
        .collect::<Vec<_>>();
    let values = users.iter().map(operation).collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
