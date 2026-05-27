use std::sync::Arc;

use telluride::{markdown::MarkdownString, markdown_format};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::{Command, command_add_category::CommandAddCategory},
    menus::common::create_buttons_menu,
    storages::{Category, CategoryStorageTrait},
    ui::CommandContext,
};

pub async fn select_category(
    target: &CommandContext,
    storage: &Arc<dyn CategoryStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&Category) -> Command,
    back_command: Option<Command>,
) -> ResponseResult<()> {
    let categories = storage.get_categories().await.unwrap_or_default();
    if categories.is_empty() {
        target
            .send_markdown_message(markdown_format!(
                "📂 No categories defined yet\\. Use {} to create one\\.",
                CommandAddCategory::default().to_command_string(true)
            ))
            .await?;
        return Ok(());
    }

    let category_list: Vec<Category> = categories
        .keys()
        .filter_map(|name| Category::from_string(name).ok())
        .collect();

    let msg = target.markdown_message(prompt).await?;
    let menu = create_categories_menu(&category_list, next_command, back_command, false);
    let keyboard = target.keyboard(menu).await;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub fn create_categories_menu(
    categories: &[Category],
    operation: impl Fn(&Category) -> Command,
    back_command: Option<Command>,
    inline: bool,
) -> Vec<Vec<crate::ui::ButtonData>> {
    let texts = categories
        .iter()
        .map(|category| format!("📁 {}", category.as_str()))
        .collect::<Vec<_>>();
    let values = categories.iter().map(operation).collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
