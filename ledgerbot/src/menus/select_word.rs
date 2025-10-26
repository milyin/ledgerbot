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
/// Automatically shows inactive buttons when at page boundaries
/// Selected words are marked with a tick (✓)
#[allow(clippy::too_many_arguments)]
pub async fn select_word<NEXT: CommandTrait, PAGE: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    prompt: impl Fn(usize, usize, usize) -> MarkdownString,
    all_words: &[String],
    selected_words: &[String],
    page: usize,
    word_command: impl Fn(&str) -> NEXT,
    page_command: impl Fn(usize) -> PAGE,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    const WORDS_PER_PAGE: usize = 20;
    let total_words = all_words.len();
    let total_pages = total_words.div_ceil(WORDS_PER_PAGE);
    let page_number = page.min(total_pages.saturating_sub(1));

    let msg = target
        .markdown_message(prompt(page_number + 1, total_pages, total_words))
        .await?;

    let menu = create_word_menu(
        all_words,
        selected_words,
        |word| word_command(word).to_command_string(false),
        page_number,
        total_pages,
        |page_num| page_command(page_num).to_command_string(false),
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
    selected_words: &[String],
    operation: impl Fn(&str) -> String,
    page_number: usize,
    total_pages: usize,
    page_command: impl Fn(usize) -> String,
    back_command: Option<impl CommandTrait>,
) -> InlineKeyboardMarkup {
    const WORDS_PER_PAGE: usize = 20;

    // Calculate page offset
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
        // Check if this word is selected and mark it with a tick
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        row.push(InlineKeyboardButton::callback(label, operation(word)));

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

    // Previous page button
    if page_number > 0 {
        // Active: call page_command with previous page number
        nav_row.push(InlineKeyboardButton::callback(
            "◀️",
            page_command(page_number - 1),
        ));
    } else {
        // On first page - inactive
        nav_row.push(InlineKeyboardButton::callback("◁", "noop"));
    }

    // Next page button
    if page_number + 1 < total_pages {
        // Active: call page_command with next page number
        nav_row.push(InlineKeyboardButton::callback(
            "▶️",
            page_command(page_number + 1),
        ));
    } else {
        // On last page - inactive
        nav_row.push(InlineKeyboardButton::callback("▷", "noop"));
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
