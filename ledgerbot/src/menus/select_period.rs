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

use crate::{menus::common::create_buttons_menu, storages::ExpenseStorageTrait};

pub async fn select_period<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn ExpenseStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&str) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let periods = storage.list_periods(target.chat.id).await;
    if periods.is_empty() {
        target
            .send_markdown_message(markdown_format!(
                "📅 No periods with expenses found yet\\. Add expenses to create periods\\."
            ))
            .await?;
        return Ok(());
    }
    let msg = target.markdown_message(prompt).await?;
    let menu = create_periods_menu(
        &periods,
        |period| next_command(period).to_command_string(false),
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

fn create_periods_menu(
    periods: &[String],
    operation: impl Fn(&str) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = periods
        .iter()
        .map(|period| format!("📅 {}", period))
        .collect::<Vec<_>>();
    let values = periods
        .iter()
        .map(|period| operation(period))
        .collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
