use std::sync::Arc;

use telluride::markdown::MarkdownString;
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::Command,
    menus::common::read_category_filter_by_index,
    storages::{Category, CategoryStorageTrait},
    ui::{ButtonData, CommandContext},
};

#[allow(clippy::too_many_arguments)]
pub async fn update_category_filter(
    target: &CommandContext,
    storage: &Arc<dyn CategoryStorageTrait>,
    category: &Category,
    idx: usize,
    prompt: impl Fn(&str) -> MarkdownString,
    button_text: &str,
    update_command: impl Fn(&str) -> Command,
    back_command: Option<Command>,
) -> ResponseResult<()> {
    let Some(pattern) =
        read_category_filter_by_index(target, storage, category, idx, back_command.clone()).await?
    else {
        return Ok(());
    };
    let msg = target.markdown_message(prompt(&pattern)).await?;
    let mut buttons = vec![vec![ButtonData::SwitchInlineQuery(
        button_text.to_string(),
        update_command(&pattern).to_command_string(false),
    )]];
    if let Some(back) = back_command {
        buttons.push(vec![ButtonData::Command("↩️ Back".to_string(), back)]);
    };
    let keyboard = target.keyboard(buttons).await;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
