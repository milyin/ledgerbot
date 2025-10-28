use std::{fmt::Display, str::FromStr};

use regex::Regex;
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    utils::command::ParseError,
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    storage::{ButtonData, pack_callback_data},
};

/// Represents a collection of words separated by '|'
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Words(Vec<String>);

impl Words {
    pub fn new(words: Vec<String>) -> Self {
        Self(words)
    }

    pub fn as_vec(&self) -> &Vec<String> {
        &self.0
    }

    /// Build a regex pattern from the words: (?i)\b(word1|word2|word3)\b
    pub fn build_pattern(&self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }
        let escaped_words: Vec<String> = self.0.iter().map(|w| regex::escape(w)).collect();
        Some(format!(r"(?i)\b({})\b", escaped_words.join("|")))
    }

    /// Parse a regex pattern back into Words
    /// Expects pattern format: (?i)\b(word1|word2|word3)\b
    /// Returns None if pattern doesn't match this format
    pub fn read_pattern(pattern: &str) -> Option<Self> {
        // Pattern to match: (?i)\b(word1|word2|word3)\b
        // We need to extract the words from between \b( and )\b
        let re = Regex::new(r"^\(\?i\)\\b\((.+)\)\\b$").ok()?;
        let captures = re.captures(pattern)?;
        let words_part = captures.get(1)?.as_str();

        // Split by | and unescape each word
        let words: Vec<String> = words_part
            .split('|')
            .map(|escaped_word| {
                // Unescape regex escapes - reverse of regex::escape()
                // regex::escape escapes: . + * ? ( ) | [ ] { } ^ $ # & - ~ \ /
                escaped_word
                    .replace(r"\.", ".")
                    .replace(r"\+", "+")
                    .replace(r"\*", "*")
                    .replace(r"\?", "?")
                    .replace(r"\(", "(")
                    .replace(r"\)", ")")
                    .replace(r"\|", "|")
                    .replace(r"\[", "[")
                    .replace(r"\]", "]")
                    .replace(r"\{", "{")
                    .replace(r"\}", "}")
                    .replace(r"\^", "^")
                    .replace(r"\$", "$")
                    .replace(r"\#", "#")
                    .replace(r"\&", "&")
                    .replace(r"\-", "-")
                    .replace(r"\~", "~")
                    .replace(r"\\", "\\")
                    .replace(r"\/", "/")
            })
            .collect();

        Some(Words::new(words))
    }
}

impl Display for Words {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("|"))
    }
}

impl FromStr for Words {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words = s.split('|').map(|w| w.trim().to_string()).collect();
        Ok(Words(words))
    }
}

impl AsRef<Vec<String>> for Words {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl AsMut<Vec<String>> for Words {
    fn as_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

impl From<Vec<String>> for Words {
    fn from(words: Vec<String>) -> Self {
        Words::new(words)
    }
}

/// Display a menu with word suggestions for filter creation
/// Words are displayed in a grid (4 words per row)
/// Handles pagination internally - pass full word list and page number
/// Automatically shows inactive buttons when at page boundaries
/// Selected words are marked with a tick (✓)
#[allow(clippy::too_many_arguments)]
pub async fn select_word<
    NEXT: CommandTrait,
    PAGE: CommandTrait,
    BACK: CommandTrait,
    APPLY: CommandTrait,
>(
    target: &CommandReplyTarget,
    prompt: impl Fn(usize, usize, usize) -> MarkdownString,
    all_words: &[String],
    selected_words: &[String],
    page: usize,
    word_command: impl Fn(&str) -> NEXT,
    page_command: impl Fn(usize) -> PAGE,
    apply_command: APPLY,
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

    // Create the menu with word buttons, navigation, and apply button
    let button_data = create_word_menu_data(
        all_words,
        selected_words,
        |word| word_command(word).to_command_string(false),
        page_number,
        total_pages,
        |page_num| page_command(page_num).to_command_string(false),
        apply_command.to_command_string(false),
        back_command.as_ref(),
    );

    // Pack all buttons (callback and inline query) into the keyboard
    let keyboard = pack_callback_data(
        &target.callback_data_storage,
        target.chat.id,
        msg.id.0,
        button_data,
    )
    .await;

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
    apply_command: String,
    back_command: Option<&impl CommandTrait>,
) -> Vec<Vec<ButtonData>> {
    const WORDS_PER_PAGE: usize = 20;

    // Calculate page offset
    let page_offset = page_number * WORDS_PER_PAGE;

    // Get words for current page
    let page_words: Vec<&String> = all_words
        .iter()
        .skip(page_offset)
        .take(WORDS_PER_PAGE)
        .collect();

    let mut buttons: Vec<Vec<ButtonData>> = Vec::new();
    let mut row: Vec<ButtonData> = Vec::new();

    // Create buttons for words on current page (4 per row)
    for word in page_words {
        // Check if this word is selected and mark it with a tick
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        row.push(ButtonData::Callback(label, operation(word)));

        if row.len() == 4 {
            buttons.push(row.clone());
            row.clear();
        }
    }

    // Add remaining buttons if any
    if !row.is_empty() {
        buttons.push(row);
    }

    // Add navigation buttons row: Prev, Next, Back, Apply
    let mut nav_row: Vec<ButtonData> = Vec::new();

    // Previous page button
    if page_number > 0 {
        // Active: call page_command with previous page number
        nav_row.push(ButtonData::Callback(
            "◀️".to_string(),
            page_command(page_number - 1),
        ));
    } else {
        // On first page - inactive
        nav_row.push(ButtonData::Callback("◁".to_string(), "noop".to_string()));
    }

    // Next page button
    if page_number + 1 < total_pages {
        // Active: call page_command with next page number
        nav_row.push(ButtonData::Callback(
            "▶️".to_string(),
            page_command(page_number + 1),
        ));
    } else {
        // On last page - inactive
        nav_row.push(ButtonData::Callback("▷".to_string(), "noop".to_string()));
    }

    // Add back button if provided
    if let Some(back) = back_command {
        nav_row.push(ButtonData::Callback(
            "↩️ Back".to_string(),
            back.to_command_string(false),
        ));
    }

    // Add apply button (switch inline query type)
    nav_row.push(ButtonData::SwitchInlineQuery(
        "✅ Apply".to_string(),
        apply_command,
    ));

    buttons.push(nav_row);

    buttons
}
