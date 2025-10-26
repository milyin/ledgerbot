use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::markdown::MarkdownString;

use crate::commands::command_trait::{CommandReplyTarget, CommandTrait};

/// Display a menu with word suggestions for filter creation
/// Words are displayed in a grid (4 words per row)
pub async fn select_word<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    prompt: MarkdownString,
    words: &[String],
    next_command: impl Fn(&str) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let msg = target.markdown_message(prompt).await?;

    let menu = create_word_menu(
        words,
        |word| next_command(word).to_command_string(false),
        back_command,
    );

    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(menu)
        .await?;

    Ok(())
}

fn create_word_menu(
    words: &[String],
    operation: impl Fn(&str) -> String,
    back_command: Option<impl CommandTrait>,
) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    // Create buttons for words (4 per row)
    for word in words {
        row.push(InlineKeyboardButton::callback(
            word,
            operation(word),
        ));

        if row.len() == 4 {
            buttons.push(row.clone());
            row.clear();
        }
    }

    // Add remaining buttons if any
    if !row.is_empty() {
        buttons.push(row);
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
