use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::markdown::MarkdownString;

use crate::commands::command_trait::{CommandReplyTarget, CommandTrait};

/// Display a menu with word suggestions for filter creation
/// Words are displayed in a grid (4 words per row)
/// Handles pagination internally - pass full word list and page number
pub async fn select_word<NEXT: CommandTrait, PREV: CommandTrait, NEXTP: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    prompt: MarkdownString,
    all_words: &[String],
    page: usize,
    next_command: impl Fn(&str) -> NEXT,
    prev_page_command: Option<PREV>,
    next_page_command: Option<NEXTP>,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let msg = target.markdown_message(prompt).await?;

    let menu = create_word_menu(
        all_words,
        page,
        |word| next_command(word).to_command_string(false),
        prev_page_command,
        next_page_command,
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
    all_words: &[String],
    page: usize,
    operation: impl Fn(&str) -> String,
    prev_page_command: Option<impl CommandTrait>,
    next_page_command: Option<impl CommandTrait>,
    back_command: Option<impl CommandTrait>,
) -> InlineKeyboardMarkup {
    const WORDS_PER_PAGE: usize = 20;

    // Calculate pagination
    let total_words = all_words.len();
    let total_pages = total_words.div_ceil(WORDS_PER_PAGE);
    let page_number = page.min(total_pages.saturating_sub(1));
    let page_offset = page_number * WORDS_PER_PAGE;

    // Get words for current page
    let page_words: Vec<&String> = all_words
        .iter()
        .skip(page_offset)
        .take(WORDS_PER_PAGE)
        .collect();

    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let mut row: Vec<InlineKeyboardButton> = Vec::new();

    // Create buttons for words on current page (4 per row)
    for word in page_words {
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

    // Add navigation buttons row: Prev, Next, Back
    let mut nav_row: Vec<InlineKeyboardButton> = Vec::new();

    // Previous page button (active or inactive)
    if let Some(prev) = prev_page_command {
        nav_row.push(InlineKeyboardButton::callback(
            "◀️",
            prev.to_command_string(false),
        ));
    } else {
        nav_row.push(InlineKeyboardButton::callback(
            "◁",
            "noop", // Inactive button
        ));
    }

    // Next page button (active or inactive)
    if let Some(next) = next_page_command {
        nav_row.push(InlineKeyboardButton::callback(
            "▶️",
            next.to_command_string(false),
        ));
    } else {
        nav_row.push(InlineKeyboardButton::callback(
            "▷",
            "noop", // Inactive button
        ));
    }

    // Add back button if provided
    if let Some(back) = back_command {
        nav_row.push(InlineKeyboardButton::callback(
            "↩️ Back",
            back.to_command_string(false),
        ));
    }

    buttons.push(nav_row);

    InlineKeyboardMarkup::new(buttons)
}
