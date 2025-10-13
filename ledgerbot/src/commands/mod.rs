pub mod categories;
pub mod expenses;
pub mod filters;
pub mod help;
pub mod report;

use chrono::NaiveDate;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId},
    utils::{
        command::{BotCommands, ParseError},
        markdown::escape,
    },
};

use std::sync::Arc;

use crate::{
    commands::{
        categories::{categories_command, category_command, remove_category_command},
        expenses::{clear_command, expense_command, list_command, parse_expense},
        filters::{add_filter_command, edit_filter_command, remove_filter_command},
        help::{help_command, start_command},
        report::report_command,
    },
    handlers::CallbackData,
    markdown,
    markdown_string::MarkdownStringSendMessage,
    parser::extract_words,
    storage_traits::{CategoryStorageTrait, StorageTrait},
};

/// Type alias for category, position, and pattern parser result
type CategoryPositionPatternResult =
    Result<(Option<String>, Option<usize>, Option<String>), ParseError>;

/// Custom parser for optional single string parameter
fn parse_optional_string(s: String) -> Result<(Option<String>,), ParseError> {
    // Take only the first line to prevent multi-line capture
    let first_line = s.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        Ok((None,))
    } else {
        Ok((Some(first_line.to_string()),))
    }
}

/// Custom parser for two optional string parameters
fn parse_two_optional_strings(s: String) -> Result<(Option<String>, Option<String>), ParseError> {
    // Take only the first line to prevent multi-line capture
    let first_line = s.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return Ok((None, None));
    }

    let parts: Vec<&str> = first_line.splitn(2, ' ').collect();
    match parts.as_slice() {
        [first] => Ok((Some(first.to_string()), None)),
        [first, second] => Ok((Some(first.to_string()), Some(second.to_string()))),
        _ => Ok((None, None)),
    }
}

/// Custom parser for category and position (for remove_filter)
fn parse_category_and_position(s: String) -> Result<(Option<String>, Option<usize>), ParseError> {
    // Take only the first line to prevent multi-line capture
    let first_line = s.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return Ok((None, None));
    }

    let parts: Vec<&str> = first_line.splitn(2, ' ').collect();
    match parts.as_slice() {
        [category] => Ok((Some(category.to_string()), None)),
        [category, position_str] => match position_str.parse::<usize>() {
            Ok(position) => Ok((Some(category.to_string()), Some(position))),
            Err(_) => Err(ParseError::IncorrectFormat(
                format!("Position must be a number, got '{}'", position_str).into(),
            )),
        },
        _ => Ok((None, None)),
    }
}

/// Custom parser for category, position, and pattern (for edit_filter)
fn parse_category_position_and_pattern(s: String) -> CategoryPositionPatternResult {
    // Take only the first line to prevent multi-line capture
    let first_line = s.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return Ok((None, None, None));
    }

    let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
    match parts.as_slice() {
        [category] => Ok((Some(category.to_string()), None, None)),
        [category, position_str] => match position_str.parse::<usize>() {
            Ok(position) => Ok((Some(category.to_string()), Some(position), None)),
            Err(_) => Err(ParseError::IncorrectFormat(
                format!("Position must be a number, got '{}'", position_str).into(),
            )),
        },
        [category, position_str, pattern] => match position_str.parse::<usize>() {
            Ok(position) => Ok((
                Some(category.to_string()),
                Some(position),
                Some(pattern.to_string()),
            )),
            Err(_) => Err(ParseError::IncorrectFormat(
                format!("Position must be a number, got '{}'", position_str).into(),
            )),
        },
        _ => Ok((None, None, None)),
    }
}

/// Bot commands
#[derive(BotCommands, Clone, Debug, PartialEq)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "start the bot")]
    Start,
    #[command(description = "display this help")]
    Help,
    #[command(description = "list expenses chronologically in input format")]
    List,
    #[command(description = "show expenses report")]
    Report,
    #[command(description = "clear all expenses")]
    Clear,
    #[command(description = "list all categories with filters in command format")]
    Categories,
    #[command(description = "clear all categories", rename = "clear_categories")]
    ClearCategories,
    #[command(
        description = "add expense category",
        rename = "add_category",
        parse_with = parse_optional_string
    )]
    AddCategory { name: Option<String> },
    #[command(
        description = "add filter to category",
        rename = "add_filter",
        parse_with = parse_two_optional_strings
    )]
    AddFilter {
        category: Option<String>,
        pattern: Option<String>,
    },
    #[command(
        description = "remove expense category",
        rename = "remove_category",
        parse_with = parse_optional_string
    )]
    RemoveCategory { name: Option<String> },
    #[command(
        description = "remove filter from category by position",
        rename = "remove_filter",
        parse_with = parse_category_and_position
    )]
    RemoveFilter {
        category: Option<String>,
        position: Option<usize>,
    },
    #[command(
        description = "edit filter in category by position",
        rename = "edit_filter",
        parse_with = parse_category_position_and_pattern
    )]
    EditFilter {
        category: Option<String>,
        position: Option<usize>,
        pattern: Option<String>,
    },
    #[command(
        description = "add expense with date, description and amount",
        parse_with = parse_expense
    )]
    Expense {
        date: Option<NaiveDate>,
        description: Option<String>,
        amount: Option<f64>,
    },
}

// Command constants as string representations
impl Command {
    pub const HELP: &'static str = "/help";
    pub const START: &'static str = "/start";
    pub const LIST: &'static str = "/list";
    pub const REPORT: &'static str = "/report";
    pub const CLEAR: &'static str = "/clear";
    pub const CATEGORIES: &'static str = "/categories";
    pub const CLEAR_CATEGORIES: &'static str = "/clear_categories";
    pub const ADD_CATEGORY: &'static str = "/add_category";
    pub const ADD_FILTER: &'static str = "/add_filter";
    pub const REMOVE_CATEGORY: &'static str = "/remove_category";
    pub const REMOVE_FILTER: &'static str = "/remove_filter";
    pub const EDIT_FILTER: &'static str = "/edit_filter";
    pub const EXPENSE: &'static str = "/expense";
}

impl From<Command> for String {
    fn from(val: Command) -> Self {
        match val {
            Command::Start => Command::START.to_string(),
            Command::Help => Command::HELP.to_string(),
            Command::List => Command::LIST.to_string(),
            Command::Report => Command::REPORT.to_string(),
            Command::Clear => Command::CLEAR.to_string(),
            Command::Categories => Command::CATEGORIES.to_string(),
            Command::ClearCategories => Command::CLEAR_CATEGORIES.to_string(),
            Command::AddCategory { name } => {
                let name_str = name.unwrap_or_else(|| "<name>".to_string());
                format!("{} {}", Command::ADD_CATEGORY, name_str)
            }
            Command::AddFilter { category, pattern } => {
                let category_str = category.unwrap_or_else(|| "<category>".to_string());
                let pattern_str = pattern.unwrap_or_else(|| "<pattern>".to_string());
                format!("{} {} {}", Command::ADD_FILTER, category_str, pattern_str)
            }
            Command::RemoveCategory { name } => {
                let name_str = name.unwrap_or_else(|| "<name>".to_string());
                format!("{} {}", Command::REMOVE_CATEGORY, name_str)
            }
            Command::RemoveFilter { category, position } => {
                let category_str = category.unwrap_or_else(|| "<category>".to_string());
                let position_str = position
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "<position>".to_string());
                format!(
                    "{} {} {}",
                    Command::REMOVE_FILTER,
                    category_str,
                    position_str
                )
            }
            Command::EditFilter {
                category,
                position,
                pattern,
            } => {
                let category_str = category.unwrap_or_else(|| "<category>".to_string());
                let position_str = position
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "<position>".to_string());
                let pattern_str = pattern.unwrap_or_else(|| "<pattern>".to_string());
                format!(
                    "{} {} {} {}",
                    Command::EDIT_FILTER,
                    category_str,
                    position_str,
                    pattern_str
                )
            }
            Command::Expense {
                date,
                description,
                amount,
            } => {
                let date_str = date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "<date>".to_string());
                let description_str = description.unwrap_or_else(|| "<description>".to_string());
                let amount_str = amount
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "<amount>".to_string());
                format!(
                    "{} {} {} {}",
                    Command::EXPENSE,
                    date_str,
                    description_str,
                    amount_str
                )
            }
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

/// Clear all categories
pub async fn clear_categories_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn CategoryStorageTrait>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    storage.clear_chat_categories(chat_id).await;

    bot.send_markdown_message(chat_id, markdown!("🗑️ All categories cleared\\!"))
        .await?;
    Ok(())
}

/// Show word suggestions for adding filters to a category
pub async fn show_filter_word_suggestions(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: Arc<dyn StorageTrait>,
    category_name: String,
) -> ResponseResult<()> {
    let expenses = storage
        .clone()
        .as_expense_storage()
        .get_chat_expenses(chat_id)
        .await;
    let categories = storage
        .clone()
        .as_category_storage()
        .get_chat_categories(chat_id)
        .await;

    // Get currently selected words from storage
    let selected_words = storage
        .clone()
        .as_filter_selection_storage()
        .get_filter_selection(chat_id, &category_name)
        .await;

    // Get current page offset
    let page_offset = storage
        .clone()
        .as_filter_page_storage()
        .get_filter_page_offset(chat_id, &category_name)
        .await;

    // Extract words from uncategorized expenses
    let words = extract_words(&expenses, &categories);

    // Pagination constants
    const WORDS_PER_PAGE: usize = 20;
    let total_words = words.len();
    let total_pages = total_words.div_ceil(WORDS_PER_PAGE);
    let current_page = page_offset / WORDS_PER_PAGE + 1;

    // Get words for current page
    let page_words: Vec<&String> = words
        .iter()
        .skip(page_offset)
        .take(WORDS_PER_PAGE)
        .collect();

    // Build selected words display
    let selected_display = if selected_words.is_empty() {
        escape("(none selected)")
    } else {
        escape(&selected_words.join(" | "))
    };

    let escaped_category = escape(&category_name);
    let text = format!(
        "💡 *Select word\\(s\\) for filter '{}'*\n\n*Selected:* {}\n\nPage {}/{} \\({} words total\\)\n\nClick words to add/remove them\\. When done, click 'Apply Filter'\\.",
        escaped_category, selected_display, current_page, total_pages, total_words
    );

    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    // Add buttons for each suggested word on current page (4 per row)
    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    for word in page_words {
        // Check if this word is selected
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        // Use CallbackData enum for type-safe callback data
        row.push(InlineKeyboardButton::callback(
            label,
            CallbackData::ToggleWord {
                category: category_name.clone(),
                word: word.clone(),
            },
        ));

        // Add row when we have 4 buttons
        if row.len() == 4 {
            buttons.push(row.clone());
            row.clear();
        }
    }

    // Add remaining buttons if any
    if !row.is_empty() {
        buttons.push(row);
    }

    // Add all control buttons in a single row: Left, Right, Apply, Back
    let mut control_row: Vec<InlineKeyboardButton> = Vec::new();

    // Previous page button (always shown, inactive if on first page)
    if page_offset > 0 {
        control_row.push(InlineKeyboardButton::callback(
            "◀️",
            CallbackData::PagePrev(category_name.clone()),
        ));
    } else {
        // Inactive button with dummy callback data
        control_row.push(InlineKeyboardButton::callback("◁", CallbackData::Noop));
    }

    // Next page button (always shown, inactive if on last page)
    if page_offset + WORDS_PER_PAGE < total_words {
        control_row.push(InlineKeyboardButton::callback(
            "▶️",
            CallbackData::PageNext(category_name.clone()),
        ));
    } else {
        // Inactive button with dummy callback data
        control_row.push(InlineKeyboardButton::callback("️▷", CallbackData::Noop));
    }

    // Apply button - puts /add_filter command with generated regexp in input box
    let apply_command = if !selected_words.is_empty() {
        // Escape each word and combine with case-insensitive OR pattern with word boundaries
        let escaped_words: Vec<String> = selected_words.iter().map(|w| regex::escape(w)).collect();
        let pattern = format!(r"(?i)\b({})\b", escaped_words.join("|"));
        Command::AddFilter {
            category: Some(category_name.clone()),
            pattern: Some(pattern),
        }
        .to_string()
    } else {
        // No words selected, just put category name
        format!("{} {} ", Command::ADD_FILTER, category_name)
    };

    control_row.push(InlineKeyboardButton::switch_inline_query_current_chat(
        "✅ Apply",
        apply_command,
    ));

    // Back button
    control_row.push(InlineKeyboardButton::callback(
        "↩️ Back",
        CallbackData::CmdAddFilter,
    ));

    buttons.push(control_row);

    let keyboard = InlineKeyboardMarkup::new(buttons);

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    bot.edit_message_reply_markup(chat_id, message_id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Execute a single command (helper function for batch processing and text message handling)
pub async fn execute_command(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn StorageTrait>,
    cmd: Command,
    silent: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Command::Start => {
            start_command(bot.clone(), msg.clone()).await?;
        }
        Command::Help => {
            help_command(bot.clone(), msg.clone()).await?;
        }
        Command::Expense {
            date,
            description,
            amount,
        } => {
            expense_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_expense_storage(),
                date,
                description,
                amount,
                silent,
            )
            .await?;
        }
        Command::List => {
            list_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_expense_storage(),
            )
            .await?;
        }
        Command::Report => {
            report_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_expense_storage(),
                storage.clone().as_category_storage(),
            )
            .await?;
        }
        Command::Clear => {
            clear_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_expense_storage(),
            )
            .await?;
        }
        Command::ClearCategories => {
            clear_categories_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
            )
            .await?;
        }
        Command::AddCategory { name } => {
            category_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
                name,
            )
            .await?;
        }
        Command::Categories => {
            categories_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
            )
            .await?;
        }
        Command::AddFilter { category, pattern } => {
            add_filter_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
                category,
                pattern,
            )
            .await?;
        }
        Command::RemoveCategory { name } => {
            remove_category_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
                name,
            )
            .await?;
        }
        Command::RemoveFilter { category, position } => {
            remove_filter_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
                category,
                position,
            )
            .await?;
        }
        Command::EditFilter {
            category,
            position,
            pattern,
        } => {
            edit_filter_command(
                bot.clone(),
                msg.clone(),
                storage.clone().as_category_storage(),
                category,
                position,
                pattern,
            )
            .await?;
        }
    }
    Ok(())
}
