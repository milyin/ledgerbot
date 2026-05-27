use std::sync::Arc;

use telluride::{markdown::MarkdownString, markdown_format};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::Command,
    storages::{Category, CategoryStorageTrait},
    ui::{ButtonData, CommandContext},
};

pub async fn update_category(
    target: &CommandContext,
    storage: &Arc<dyn CategoryStorageTrait>,
    category: &Category,
    prompt: MarkdownString,
    button_text: &str,
    update_command: Command,
    back_command: Option<Command>,
) -> ResponseResult<()> {
    let categories = storage.get_categories().await.unwrap_or_default();
    if !categories.contains_key(category.as_str()) {
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
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let mut buttons = vec![vec![ButtonData::SwitchInlineQuery(
        button_text.to_string(),
        update_command.to_command_string(false),
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
