use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::InlineKeyboardMarkup,
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
};

use crate::{
    menus::common::{create_buttons_menu, read_category_filters_list},
    storage_traits::CategoryStorageTrait,
};

pub async fn select_category_filter<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    category_name: &str,
    prompt: MarkdownString,
    next_command: impl Fn(usize, &str) -> Option<NEXT>,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let filters =
        read_category_filters_list(target, storage, category_name, back_command.clone()).await?;
    if filters.is_empty() {
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let menu = create_category_filters_menu(
        &filters,
        |idx, pattern| next_command(idx, pattern).map(|cmd| cmd.to_command_string(false)),
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
    operation: impl Fn(usize, &str) -> Option<String>,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    // Filter out items where operation returns None
    let items: Vec<(String, String)> = filters
        .iter()
        .enumerate()
        .filter_map(|(idx, pattern)| {
            operation(idx, pattern).map(|value| (format!("{}. {}", idx, pattern), value))
        })
        .collect();

    let texts: Vec<String> = items.iter().map(|(text, _)| text.clone()).collect();
    let values: Vec<String> = items.iter().map(|(_, value)| value.clone()).collect();

    // use create_menu
    create_buttons_menu(&texts, &values, back_command, inline)
}
