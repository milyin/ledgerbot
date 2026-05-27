use std::{fmt::Display, str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};
use telluride::markdown::MarkdownString;
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    utils::command::ParseError,
};

use crate::{
    commands::Command,
    ui::{ButtonData, CommandContext},
};

/// Represents a collection of words separated by '|'
#[derive(Debug, Clone, PartialEq, Default, Eq, Hash, Serialize, Deserialize)]
pub struct Words(Vec<String>);

impl Words {
    pub fn new(words: Vec<String>) -> Self {
        Self(words)
    }

    pub fn as_vec(&self) -> &Vec<String> {
        &self.0
    }

    pub fn build_pattern(&self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }
        let escaped_words: Vec<String> = self.0.iter().map(|w| regex::escape(w)).collect();
        Some(format!(r"(?i)\b({})\b", escaped_words.join("|")))
    }

    pub fn read_pattern(pattern: &str) -> Option<Self> {
        let re = Regex::new(r"^\(\?i\)\\b\((.+)\)\\b$").ok()?;
        let captures = re.captures(pattern)?;
        let words_part = captures.get(1)?.as_str();

        let words: Vec<String> = words_part
            .split('|')
            .map(|escaped_word| {
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

impl From<Words> for Vec<String> {
    fn from(val: Words) -> Self {
        val.0
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn select_word(
    target: &CommandContext,
    prompt: impl Fn(usize, usize, usize) -> MarkdownString,
    all_words: &[String],
    selected_words: &[String],
    page: usize,
    word_command: impl Fn(&str) -> Command,
    page_command: impl Fn(usize) -> Command,
    apply_command: Command,
    back_command: Option<Command>,
) -> ResponseResult<()> {
    const WORDS_PER_PAGE: usize = 20;
    let total_words = all_words.len();
    let total_pages = total_words.div_ceil(WORDS_PER_PAGE);
    let page_number = page.min(total_pages.saturating_sub(1));

    let msg = target
        .markdown_message(prompt(page_number + 1, total_pages, total_words))
        .await?;

    let button_data = create_word_menu_data(
        all_words,
        selected_words,
        word_command,
        page_number,
        total_pages,
        page_command,
        apply_command.to_command_string(false),
        back_command,
    );

    let keyboard = target.keyboard(button_data).await;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn create_word_menu_data(
    all_words: &[String],
    selected_words: &[String],
    operation: impl Fn(&str) -> Command,
    page_number: usize,
    total_pages: usize,
    page_command: impl Fn(usize) -> Command,
    apply_command: String,
    back_command: Option<Command>,
) -> Vec<Vec<ButtonData>> {
    const WORDS_PER_PAGE: usize = 20;

    let page_offset = page_number * WORDS_PER_PAGE;
    let page_words: Vec<&String> = all_words
        .iter()
        .skip(page_offset)
        .take(WORDS_PER_PAGE)
        .collect();

    let mut buttons: Vec<Vec<ButtonData>> = Vec::new();
    let mut row: Vec<ButtonData> = Vec::new();

    for word in page_words {
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        row.push(ButtonData::Command(label, operation(word)));

        if row.len() == 4 {
            buttons.push(row.clone());
            row.clear();
        }
    }

    if !row.is_empty() {
        buttons.push(row);
    }

    let mut nav_row: Vec<ButtonData> = Vec::new();

    if page_number > 0 {
        nav_row.push(ButtonData::Command(
            "◀️".to_string(),
            page_command(page_number - 1),
        ));
    } else {
        nav_row.push(ButtonData::RawCallback("◁".to_string(), "noop".to_string()));
    }

    if page_number + 1 < total_pages {
        nav_row.push(ButtonData::Command(
            "▶️".to_string(),
            page_command(page_number + 1),
        ));
    } else {
        nav_row.push(ButtonData::RawCallback("▷".to_string(), "noop".to_string()));
    }

    if let Some(back) = back_command {
        nav_row.push(ButtonData::Command("↩️ Back".to_string(), back));
    }

    nav_row.push(ButtonData::SwitchInlineQuery(
        "✅ Apply".to_string(),
        apply_command,
    ));

    buttons.push(nav_row);
    buttons
}
