use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    markdown_format,
};

use crate::storages::{ShareStorageTrait, ShareUsername};

pub async fn update_share<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn ShareStorageTrait>,
    username: &ShareUsername,
    prompt: MarkdownString,
    button_text: &str,
    update_command: NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let shares = storage
        .get_shares()
        .await
        .unwrap_or_default();
    if !shares.contains(username) {
        let msg = target
            .markdown_message(markdown_format!(
                "❌ Username `{}` is not in the share list",
                username.as_str()
            ))
            .await?;
        if let Some(back) = back_command {
            let menu = vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]];
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(teloxide::types::InlineKeyboardMarkup::new(menu))
                .await?;
        }
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let mut buttons = vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            button_text,
            update_command.to_command_string(false),
        ),
    ]];
    if let Some(back) = back_command {
        buttons.push(vec![InlineKeyboardButton::callback(
            "↩️ Back",
            back.to_command_string(false),
        )]);
    };
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;
    Ok(())
}
