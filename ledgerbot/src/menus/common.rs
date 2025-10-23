use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::markdown_format;

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait},
    storage_traits::CategoryStorageTrait,
};

pub fn create_buttons_menu(
    titles: &[String],
    values: &[String],
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = titles
        .iter()
        .zip(values.iter())
        .map(|(text, value)| {
            if inline {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    text,
                    value.clone(),
                )]
            } else {
                vec![InlineKeyboardButton::callback(text, value.clone())]
            }
        })
        .collect();
    if let Some(back) = back_command {
        buttons.push(vec![InlineKeyboardButton::callback(
            "↩️ Back",
            back.to_command_string(false),
        )]);
    }
    InlineKeyboardMarkup::new(buttons)
}

pub async fn read_category_filters_list(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    back_command: Option<impl CommandTrait>,
) -> ResponseResult<Vec<String>> {
    let categories = storage.get_chat_categories(target.chat.id).await;
    let Some(filters) = categories.get(name) else {
        let msg = target
            .markdown_message(markdown_format!("❌ Category `{}` does not exist", name))
            .await?;
        if let Some(back) = back_command {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(Vec::new());
    };
    if filters.is_empty() {
        let msg = target
            .markdown_message(markdown_format!(
                "📂 Category `{}` has no filters defined yet\\.",
                name
            ))
            .await?;
        if let Some(back) = back_command {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(Vec::new());
    }
    Ok(filters.clone())
}

pub async fn read_category_filter_by_index(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    idx: usize,
    back_command: Option<impl CommandTrait>,
) -> ResponseResult<Option<String>> {
    let filters = read_category_filters_list(target, storage, name, back_command.clone()).await?;
    if filters.is_empty() {
        return Ok(None);
    };
    if idx >= filters.len() {
        let msg = target
            .markdown_message(markdown_format!("❌ Invalid filter position `{}`", idx))
            .await?;
        if let Some(back) = back_command {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(None);
    }
    Ok(Some(filters[idx].clone()))
}
