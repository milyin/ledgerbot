use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
};

use crate::storages::ExpenseStorageTrait;

pub async fn select_period<NEXT: CommandTrait, BACK: CommandTrait, NEWPERIOD: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn ExpenseStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&str) -> NEXT,
    back_command: Option<BACK>,
    new_period_command: Option<NEWPERIOD>,
) -> ResponseResult<()> {
    let periods = storage.list_periods(target.chat.id).await;
    let msg = target.markdown_message(prompt).await?;
    if periods.is_empty() && new_period_command.is_none() {
        target
            .send_markdown_message(MarkdownString::from(
                "📅 No periods available\\. Please add expenses to create periods\\.",
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
    periods: &[String],
    operation: impl Fn(&str) -> String,
    back_command: Option<impl CommandTrait>,
    new_period_command: Option<String>,
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

    // Create the basic menu with period buttons
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = texts
        .iter()
        .zip(values.iter())
        .map(|(text, value)| {
            if inline {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    text,
                    value.clone(),
                )]
            } else {
                vec![InlineKeyboardButton::callback(text, value.clone())]
            }
        })
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
