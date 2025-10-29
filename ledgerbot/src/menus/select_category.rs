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
    commands::command_add_category::CommandAddCategory, menus::common::create_buttons_menu,
    storage_traits::CategoryStorageTrait,
};

pub async fn select_category<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&str) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let categories = storage
        .get_chat_categories(target.chat.id)
        .await
        .unwrap_or_default();
    if categories.is_empty() {
        target
            .send_markdown_message(markdown_format!(
                "📂 No categories defined yet\\. Use {} to create one\\.",
                CommandAddCategory::default().to_command_string(true)
            ))
            .await?;
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let menu = create_categories_menu(
        &categories.keys().cloned().collect::<Vec<_>>(),
        |name| next_command(name).to_command_string(false),
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

fn create_categories_menu(
    categories: &[String],
    operation: impl Fn(&str) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = categories
        .iter()
        .map(|name| format!("📁 {}", name))
        .collect::<Vec<_>>();
    let values = categories
        .iter()
        .map(|name| operation(name))
        .collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
