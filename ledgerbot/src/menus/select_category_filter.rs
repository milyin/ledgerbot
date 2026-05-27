use std::sync::Arc;

use telluride::markdown::MarkdownString;
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::Command,
    menus::common::{create_buttons_menu, read_category_filters_list},
    storages::{Category, CategoryStorageTrait},
    ui::CommandContext,
};

pub async fn select_category_filter(
    target: &CommandContext,
    storage: &Arc<dyn CategoryStorageTrait>,
    category: &Category,
    prompt: MarkdownString,
    next_command: impl Fn(usize, &str) -> Option<Command>,
    back_command: Option<Command>,
) -> ResponseResult<()> {
    let filters =
        read_category_filters_list(target, storage, category, back_command.clone()).await?;
    if filters.is_empty() {
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let menu = create_category_filters_menu(&filters, next_command, back_command, false);
    let keyboard = target.keyboard(menu).await;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub fn create_category_filters_menu(
    filters: &[String],
    operation: impl Fn(usize, &str) -> Option<Command>,
    back_command: Option<Command>,
    inline: bool,
) -> Vec<Vec<crate::ui::ButtonData>> {
    let items: Vec<(String, Command)> = filters
        .iter()
        .enumerate()
        .filter_map(|(idx, pattern)| {
            operation(idx, pattern).map(|value| (format!("{}. {}", idx, pattern), value))
        })
        .collect();

    let texts: Vec<String> = items.iter().map(|(text, _)| text.clone()).collect();
    let values: Vec<Command> = items.into_iter().map(|(_, value)| value).collect();
    create_buttons_menu(&texts, &values, back_command, inline)
}
