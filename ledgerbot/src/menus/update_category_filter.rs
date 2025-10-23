use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::markdown::MarkdownString;

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait},
    menus::common::read_category_filter_by_index,
    storage_traits::CategoryStorageTrait,
};

pub async fn update_category_filter<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    idx: usize,
    prompt: impl Fn(&str) -> MarkdownString,
    button_text: &str,
    update_command: impl Fn(&str) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let Some(pattern) =
        read_category_filter_by_index(target, storage, name, idx, back_command.clone()).await?
    else {
        return Ok(());
    };
    let msg = target.markdown_message(prompt(&pattern)).await?;
    let mut buttons = vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            button_text,
            update_command(&pattern).to_command_string(false),
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
