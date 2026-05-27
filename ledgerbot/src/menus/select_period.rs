use std::sync::Arc;

use telluride::markdown::MarkdownString;
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
};

use crate::{
    commands::Command,
    storages::{ExpensePeriod, ExpenseStorageReadTrait},
    ui::{ButtonData, CommandContext},
};

pub async fn select_period(
    target: &CommandContext,
    stores: &Arc<dyn ExpenseStorageReadTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&ExpensePeriod) -> Command,
    back_command: Option<Command>,
    new_period_command: Option<Command>,
) -> ResponseResult<()> {
    let periods = stores.list_periods().await;
    let msg = target.markdown_message(prompt).await?;
    if periods.is_empty() && new_period_command.is_none() {
        target
            .send_markdown_message(telluride::markdown_string!(
                "📅 No periods available\\. Please add expenses to create periods\\."
            ))
            .await?;
        return Ok(());
    }
    let menu = create_periods_menu(
        &periods,
        next_command,
        back_command,
        new_period_command,
        false,
    );
    let keyboard = target.keyboard(menu).await;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

fn create_periods_menu(
    periods: &[ExpensePeriod],
    operation: impl Fn(&ExpensePeriod) -> Command,
    back_command: Option<Command>,
    new_period_command: Option<Command>,
    inline: bool,
) -> Vec<Vec<ButtonData>> {
    let texts = periods
        .iter()
        .map(|period| format!("📅 {}", period))
        .collect::<Vec<_>>();
    let values = periods.iter().map(operation).collect::<Vec<_>>();

    let period_buttons: Vec<ButtonData> = texts
        .iter()
        .zip(values.iter())
        .map(|(text, value)| {
            if inline {
                ButtonData::SwitchInlineQuery(text.clone(), value.to_command_string(false))
            } else {
                ButtonData::Command(text.clone(), value.clone())
            }
        })
        .collect();

    let mut buttons: Vec<Vec<ButtonData>> = period_buttons
        .chunks(4)
        .map(|chunk| chunk.to_vec())
        .collect();

    if let Some(command) = new_period_command {
        buttons.push(vec![ButtonData::SwitchInlineQuery(
            "➕ New Period".to_string(),
            command.to_command_string(false),
        )]);
    }

    if let Some(back) = back_command {
        buttons.push(vec![ButtonData::Command("↩️ Back".to_string(), back)]);
    }

    buttons
}
