use std::sync::Arc;

use teloxide::{payloads::EditMessageReplyMarkupSetters, prelude::{Requester, ResponseResult}, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use yoroolbot::{markdown::MarkdownString, markdown_format};

use crate::{commands::command_trait::{CommandReplyTarget, CommandTrait}, storage_traits::CategoryStorageTrait};

pub async fn update_category<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    prompt: MarkdownString,
    button_text: &str,
    update_command: NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let categories = storage.get_chat_categories(target.chat.id).await;
    if categories.get(name).is_none() {
        let msg = target
            .markdown_message(markdown_format!("❌ Category `{}` does not exist", name))
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