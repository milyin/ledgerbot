use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::InlineKeyboardButton,
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    storage::pack_callback_data,
};

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
    apply_command: impl Fn() -> String,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    const WORDS_PER_PAGE: usize = 20;
    let total_words = all_words.len();
    let total_pages = total_words.div_ceil(WORDS_PER_PAGE);
    let page_number = page.min(total_pages.saturating_sub(1));

    // Send the message first
    let msg = target
        .markdown_message(prompt(page_number + 1, total_pages, total_words))
        .await?;

    // Create the menu with word buttons and navigation
    let button_data = create_word_menu_data(
        all_words,
        selected_words,
        |word| word_command(word).to_command_string(false),
        page_number,
        total_pages,
        |page_num| page_command(page_num).to_command_string(false),
        back_command.as_ref(),
    );

    // Pack the callback data for word and navigation buttons
    let mut keyboard = pack_callback_data(
        &target.callback_data_storage,
        target.chat.id,
        msg.id.0,
        button_data,
    )
    .await;

    // Add Apply button to the last row (navigation row with Prev, Next, Back)
    let apply_cmd = apply_command();
    if let Some(last_row) = keyboard.inline_keyboard.last_mut() {
        last_row.push(InlineKeyboardButton::switch_inline_query_current_chat(
            "✅ Apply",
            apply_cmd,
        ));
    }

    // Attach the keyboard to the message
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

fn create_word_menu_data(
    all_words: &[String],
    selected_words: &[String],
    operation: impl Fn(&str) -> String,
    page_number: usize,
    total_pages: usize,
    page_command: impl Fn(usize) -> String,
    back_command: Option<&impl CommandTrait>,
) -> Vec<Vec<(String, String)>> {
    const WORDS_PER_PAGE: usize = 20;

    // Calculate page offset
    let page_offset = page_number * WORDS_PER_PAGE;

    // Get words for current page
    let page_words: Vec<&String> = all_words
        .iter()
        .skip(page_offset)
        .take(WORDS_PER_PAGE)
        .collect();

    let mut buttons: Vec<Vec<(String, String)>> = Vec::new();
    let mut row: Vec<(String, String)> = Vec::new();

    // Create buttons for words on current page (4 per row)
    for word in page_words {
        // Check if this word is selected and mark it with a tick
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        row.push((label, operation(word)));

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
    let mut nav_row: Vec<(String, String)> = Vec::new();

    // Previous page button
    if page_number > 0 {
        // Active: call page_command with previous page number
        nav_row.push(("◀️".to_string(), page_command(page_number - 1)));
    } else {
        // On first page - inactive
        nav_row.push(("◁".to_string(), "noop".to_string()));
    }

    // Next page button
    if page_number + 1 < total_pages {
        // Active: call page_command with next page number
        nav_row.push(("▶️".to_string(), page_command(page_number + 1)));
    } else {
        // On last page - inactive
        nav_row.push(("▷".to_string(), "noop".to_string()));
    }

    // Add back button if provided
    if let Some(back) = back_command {
        nav_row.push(("↩️ Back".to_string(), back.to_command_string(false)));
    }

    buttons.push(nav_row);

    buttons
}
