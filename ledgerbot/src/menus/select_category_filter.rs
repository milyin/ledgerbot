use std::sync::Arc;

use teloxide::{payloads::EditMessageReplyMarkupSetters, prelude::{Requester, ResponseResult}, types::InlineKeyboardMarkup};
use yoroolbot::markdown::MarkdownString;

use crate::{commands::command_trait::{CommandReplyTarget, CommandTrait}, menus::common::{create_buttons_menu, read_category_filters_list}, storage_traits::CategoryStorageTrait};

pub async fn select_category_filter<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    category_name: &str,
    prompt: MarkdownString,
    next_command: impl Fn(usize) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let filters = read_category_filters_list(target, storage, category_name, back_command.clone()).await?;
    if filters.is_empty() {
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let menu = create_category_filters_menu(
        &filters,
        |idx| next_command(idx).to_command_string(false),
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


pub fn create_category_filters_menu(
    filters: &[String],
    operation: impl Fn(usize) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = filters
        .iter()
        .enumerate()
        .map(|(idx, pattern)| format!("{}. {}", idx, pattern))
        .collect::<Vec<_>>();
    let values = filters
        .iter()
        .enumerate()
        .map(|(idx, _)| operation(idx))
        .collect::<Vec<_>>();
    // use create_menu
    create_buttons_menu(&texts, &values, back_command, inline)
}
