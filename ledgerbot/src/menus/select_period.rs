use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    markdown_string,
};

use crate::storages::{ExpensePeriod, ExpenseStorageTrait};

pub async fn select_period<NEXT: CommandTrait, BACK: CommandTrait, NEWPERIOD: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn ExpenseStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&ExpensePeriod) -> NEXT,
    back_command: Option<BACK>,
    new_period_command: Option<NEWPERIOD>,
) -> ResponseResult<()> {
    let periods = storage.list_periods(target.chat.id).await;
    let msg = target.markdown_message(prompt).await?;
    if periods.is_empty() && new_period_command.is_none() {
        target
            .send_markdown_message(markdown_string!(
                "📅 No periods available\\. Please add expenses to create periods\\."
            ))
            .await?;
        return Ok(());
    }
    let menu = create_periods_menu(
        &periods,
        |period| next_command(period).to_command_string(false),
        back_command,
        new_period_command.map(|c| c.to_command_string(false)),
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
    periods: &[ExpensePeriod],
    operation: impl Fn(&ExpensePeriod) -> String,
    back_command: Option<impl CommandTrait>,
    new_period_command: Option<String>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = periods
        .iter()
        .map(|period| format!("📅 {}", period))
        .collect::<Vec<_>>();
    let values = periods.iter().map(operation).collect::<Vec<_>>();

    // Create the basic menu with period buttons (4 per row)
    let period_buttons: Vec<InlineKeyboardButton> = texts
        .iter()
        .zip(values.iter())
        .map(|(text, value)| {
            if inline {
                InlineKeyboardButton::switch_inline_query_current_chat(text, value.clone())
            } else {
                InlineKeyboardButton::callback(text, value.clone())
            }
        })
        .collect();

    // Arrange buttons in rows of 4
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = period_buttons
        .chunks(4)
        .map(|chunk| chunk.to_vec())
        .collect();

    // Add new period button if command provided
    if let Some(command) = new_period_command {
        buttons.push(vec![
            InlineKeyboardButton::switch_inline_query_current_chat("➕ New Period", command),
        ]);
    }

    // Add back button if provided
    if let Some(back) = back_command {
        buttons.push(vec![InlineKeyboardButton::callback(
            "↩️ Back",
            back.to_command_string(false),
        )]);
    }

    InlineKeyboardMarkup::new(buttons)
}
