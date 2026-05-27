use std::sync::Arc;

use telluride::{markdown::MarkdownString, markdown_format, markdown_string};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::Command,
    storages::{Category, CategoryStorageTrait, StorageReadonly},
    ui::{ButtonData, CommandContext},
};

pub fn create_buttons_menu(
    titles: &[String],
    values: &[Command],
    back_command: Option<Command>,
    inline: bool,
) -> Vec<Vec<ButtonData>> {
    let mut buttons: Vec<Vec<ButtonData>> = titles
        .iter()
        .zip(values.iter())
        .map(|(text, value)| {
            if inline {
                vec![ButtonData::SwitchInlineQuery(
                    text.clone(),
                    value.to_command_string(false),
                )]
            } else {
                vec![ButtonData::Command(text.clone(), value.clone())]
            }
        })
        .collect();
    if let Some(back) = back_command {
        buttons.push(vec![ButtonData::Command("↩️ Back".to_string(), back)]);
    }
    buttons
}

pub async fn read_category_filters_list(
    target: &CommandContext,
    storage: &Arc<dyn CategoryStorageTrait>,
    category: &Category,
    back_command: Option<Command>,
) -> ResponseResult<Vec<String>> {
    let categories = storage.get_categories().await.unwrap_or_default();
    let Some(filters) = categories.get(category.as_str()) else {
        let msg = target
            .markdown_message(markdown_format!(
                "❌ Category `{}` does not exist",
                category.as_str()
            ))
            .await?;
        if let Some(back) = back_command {
            let menu = target
                .keyboard(vec![vec![ButtonData::Command("↩️ Back".to_string(), back)]])
                .await;
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
                category.as_str()
            ))
            .await?;
        if let Some(back) = back_command {
            let menu = target
                .keyboard(vec![vec![ButtonData::Command("↩️ Back".to_string(), back)]])
                .await;
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
    target: &CommandContext,
    storage: &Arc<dyn CategoryStorageTrait>,
    category: &Category,
    idx: usize,
    back_command: Option<Command>,
) -> ResponseResult<Option<String>> {
    let filters =
        read_category_filters_list(target, storage, category, back_command.clone()).await?;
    if filters.is_empty() {
        return Ok(None);
    };
    if idx >= filters.len() {
        let msg = target
            .markdown_message(markdown_format!("❌ Invalid filter position `{}`", idx))
            .await?;
        if let Some(back) = back_command {
            let menu = target
                .keyboard(vec![vec![ButtonData::Command("↩️ Back".to_string(), back)]])
                .await;
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

pub fn make_follow_status_message(storage: &Arc<StorageReadonly>) -> MarkdownString {
    if let Some(external_chat_id) = storage.external_chat_id() {
        markdown_format!(
            "👁 *Note*: You are viewing expenses for chat ID `{}`\\.\n\n",
            external_chat_id.0
        )
    } else {
        markdown_string!("")
    }
}

pub async fn show_follow_status_message(
    target: &CommandContext,
    storage: &Arc<StorageReadonly>,
) -> ResponseResult<()> {
    if let Some(external_chat_id) = storage.external_chat_id() {
        target
            .send_markdown_message(markdown_format!(
                "👁 *Note*: You are viewing expenses for chat ID `{}`\\.",
                external_chat_id.0
            ))
            .await?;
    }
    Ok(())
}
