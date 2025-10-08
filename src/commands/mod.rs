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

use crate::{
    commands::{
        categories::{categories_command, category_command, remove_category_command},
        expenses::{clear_command, expense_command, list_command, parse_expense},
        filters::{add_filter_command, remove_filter_command},
        help::{help_command, start_command},
        report::report_command,
    },
    handlers::CallbackData,
    parser::extract_words,
    storage::{
        CategoryStorage, ExpenseStorage, FilterPageStorage, FilterSelectionStorage,
        get_chat_categories, get_chat_expenses, get_filter_page_offset, get_filter_selection,
    },
};

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
        description = "remove filter from category",
        rename = "remove_filter",
        parse_with = parse_two_optional_strings
    )]
    RemoveFilter {
        category: Option<String>,
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

/// Clear all categories
pub async fn clear_categories_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    crate::storage::clear_chat_categories(&category_storage, chat_id).await;

    bot.send_message(chat_id, "🗑️ All categories cleared!")
        .await?;
    Ok(())
}

/// Show word suggestions for adding filters to a category
pub async fn show_filter_word_suggestions(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    storage: ExpenseStorage,
    category_storage: CategoryStorage,
    filter_selection_storage: FilterSelectionStorage,
    filter_page_storage: FilterPageStorage,
    category_name: String,
) -> ResponseResult<()> {
    let expenses = get_chat_expenses(&storage, chat_id).await;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    // Get currently selected words from storage
    let selected_words =
        get_filter_selection(&filter_selection_storage, chat_id, &category_name).await;

    // Get current page offset
    let page_offset = get_filter_page_offset(&filter_page_storage, chat_id, &category_name).await;

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
        control_row.push(InlineKeyboardButton::callback(
            "◁",
            CallbackData::Noop,
        ));
    }

    // Next page button (always shown, inactive if on last page)
    if page_offset + WORDS_PER_PAGE < total_words {
        control_row.push(InlineKeyboardButton::callback(
            "▶️",
            CallbackData::PageNext(category_name.clone()),
        ));
    } else {
        // Inactive button with dummy callback data
        control_row.push(InlineKeyboardButton::callback(
            "️▷",
            CallbackData::Noop,
        ));
    }

    // Apply button - puts /add_filter command with generated regexp in input box
    let apply_command = if !selected_words.is_empty() {
        // Escape each word and combine with case-insensitive OR pattern with word boundaries
        let escaped_words: Vec<String> = selected_words.iter().map(|w| regex::escape(w)).collect();
        let pattern = format!(r"(?i)\b({})\b", escaped_words.join("|"));
        format!("/add_filter {} {}", category_name, pattern)
    } else {
        // No words selected, just put category name
        format!("/add_filter {} ", category_name)
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
    storage: crate::storage::ExpenseStorage,
    category_storage: crate::storage::CategoryStorage,
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
                storage.clone(),
                date,
                description,
                amount,
                silent,
            )
            .await?;
        }
        Command::List => {
            list_command(bot.clone(), msg.clone(), storage.clone()).await?;
        }
        Command::Report => {
            report_command(
                bot.clone(),
                msg.clone(),
                storage.clone(),
                category_storage.clone(),
            )
            .await?;
        }
        Command::Clear => {
            clear_command(bot.clone(), msg.clone(), storage.clone()).await?;
        }
        Command::ClearCategories => {
            clear_categories_command(bot.clone(), msg.clone(), category_storage.clone()).await?;
        }
        Command::AddCategory { name } => {
            category_command(bot.clone(), msg.clone(), category_storage.clone(), name).await?;
        }
        Command::Categories => {
            categories_command(bot.clone(), msg.clone(), category_storage.clone()).await?;
        }
        Command::AddFilter { category, pattern } => {
            add_filter_command(
                bot.clone(),
                msg.clone(),
                category_storage.clone(),
                category,
                pattern,
            )
            .await?;
        }
        Command::RemoveCategory { name } => {
            remove_category_command(bot.clone(), msg.clone(), category_storage.clone(), name)
                .await?;
        }
        Command::RemoveFilter { category, pattern } => {
            remove_filter_command(
                bot.clone(),
                msg.clone(),
                category_storage.clone(),
                category,
                pattern,
            )
            .await?;
        }
    }
    Ok(())
}
