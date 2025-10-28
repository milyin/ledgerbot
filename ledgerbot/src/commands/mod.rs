pub mod command_add_category;
pub mod command_add_expense;
pub mod command_add_filter;
pub mod command_add_words_filter;
pub mod command_categories;
pub mod command_clear_categories;
pub mod command_clear_expenses;
pub mod command_edit_filter;
pub mod command_help;
pub mod command_list;
pub mod command_remove_category;
pub mod command_remove_filter;
pub mod command_report;
pub mod command_start;
pub mod expenses;
pub mod filters;
pub mod report;

use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{Chat, InlineKeyboardButton, InlineKeyboardMarkup, MessageId},
    utils::{command::BotCommands, markdown::escape},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    storage::pack_callback_data,
};

use crate::{
    commands::{
        command_add_category::CommandAddCategory, command_add_expense::CommandAddExpense,
        command_add_filter::CommandAddFilter, command_add_words_filter::CommandAddWordsFilter,
        command_categories::CommandCategories, command_clear_categories::CommandClearCategories,
        command_clear_expenses::CommandClearExpenses, command_edit_filter::CommandEditFilter,
        command_help::CommandHelp, command_list::CommandList,
        command_remove_category::CommandRemoveCategory, command_remove_filter::CommandRemoveFilter,
        command_report::CommandReport, command_start::CommandStart,
    },
    handlers::CallbackData,
    storage_traits::StorageTrait,
    utils::extract_words::extract_words,
};

/// Bot commands
#[derive(BotCommands, Clone, Debug, PartialEq)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(
        description = "start the bot",
        parse_with = CommandStart::parse_arguments
    )]
    Start(CommandStart),
    #[command(
        description = "display this help",
        parse_with = CommandHelp::parse_arguments
    )]
    Help(CommandHelp),
    #[command(
        description = "list expenses chronologically in input format",
        parse_with = CommandList::parse_arguments
    )]
    List(CommandList),
    #[command(
        description = "show expenses report",
        parse_with = CommandReport::parse_arguments
    )]
    Report(CommandReport),
    #[command(
        description = "clear all expenses",
        parse_with = CommandClearExpenses::parse_arguments
    )]
    Clear(CommandClearExpenses),
    #[command(
        description = "list all categories with filters in command format",
        parse_with = CommandCategories::parse_arguments
    )]
    Categories(CommandCategories),
    #[command(
        description = "clear all categories",
        rename = "clear_categories",
        parse_with = CommandClearCategories::parse_arguments
    )]
    ClearCategories(CommandClearCategories),
    #[command(
        description = "add expense category",
        rename = "add_category",
        parse_with = CommandAddCategory::parse_arguments
    )]
    AddCategory(CommandAddCategory),
    #[command(
        description = "add filter to category",
        rename = "add_filter",
        parse_with = CommandAddFilter::parse_arguments
    )]
    AddFilter(CommandAddFilter),
    #[command(
        description = "remove expense category",
        rename = "remove_category",
        parse_with = CommandRemoveCategory::parse_arguments
    )]
    RemoveCategory(CommandRemoveCategory),
    #[command(
        description = "remove filter from category by position",
        rename = "remove_filter",
        parse_with = CommandRemoveFilter::parse_arguments
    )]
    RemoveFilter(CommandRemoveFilter),
    #[command(
        description = "edit filter in category by position",
        rename = "edit_filter",
        parse_with = CommandEditFilter::parse_arguments
    )]
    EditFilter(CommandEditFilter),
    #[command(
        description = "add expense with explicit date, description and amount",
        rename = "add_expense",
        parse_with = CommandAddExpense::parse_arguments
    )]
    AddExpense(CommandAddExpense),
    #[command(
        description = "add filter to category (new implementation)",
        rename = "add_words_filter",
        parse_with = CommandAddWordsFilter::parse_arguments
    )]
    AddWordsFilter(CommandAddWordsFilter),
}

// Command constants as string representations
impl Command {
    pub const ADD_FILTER: &'static str = "/add_filter";
}

impl From<Command> for String {
    fn from(val: Command) -> Self {
        match val {
            Command::Start(start) => start.to_command_string(true),
            Command::Help(help) => help.to_command_string(true),
            Command::List(list) => list.to_command_string(true),
            Command::Report(report) => report.to_command_string(true),
            Command::Clear(clear) => clear.to_command_string(true),
            Command::Categories(categories) => categories.to_command_string(true),
            Command::ClearCategories(clear_categories) => clear_categories.to_command_string(true),
            Command::AddCategory(add_category) => add_category.to_command_string(true),
            Command::AddFilter(add_filter) => add_filter.to_command_string(true),
            Command::RemoveCategory(remove_category) => remove_category.to_command_string(true),
            Command::RemoveFilter(remove_filter) => remove_filter.to_command_string(true),
            Command::EditFilter(edit_filter) => edit_filter.to_command_string(true),
            Command::AddExpense(add_expense) => add_expense.to_command_string(true),
            Command::AddWordsFilter(add_filter2) => add_filter2.to_command_string(true),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
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
        .await
        .unwrap_or_default();

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

    // Build button data as rows of (label, callback_data) tuples
    let mut button_rows: Vec<Vec<(String, String)>> = Vec::new();

    // Add buttons for each suggested word on current page (4 per row)
    let mut row: Vec<(String, String)> = Vec::new();
    for word in page_words {
        // Check if this word is selected
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        // Use CallbackData enum for type-safe callback data
        let callback_data: String = CallbackData::ToggleWord {
            category: category_name.clone(),
            word: word.clone(),
        }
        .into();

        row.push((label, callback_data));

        // Add row when we have 4 buttons
        if row.len() == 4 {
            button_rows.push(row.clone());
            row.clear();
        }
    }

    // Add remaining buttons if any
    if !row.is_empty() {
        button_rows.push(row);
    }

    // Add all control buttons in a single row: Left, Right, Apply, Back
    let mut control_row: Vec<(String, String)> = Vec::new();

    // Previous page button (always shown, inactive if on first page)
    if page_offset > 0 {
        control_row.push((
            "◀️".to_string(),
            CallbackData::PagePrev(category_name.clone()).into(),
        ));
    } else {
        // Inactive button with dummy callback data
        control_row.push(("◁".to_string(), CallbackData::Noop.into()));
    }

    // Next page button (always shown, inactive if on last page)
    if page_offset + WORDS_PER_PAGE < total_words {
        control_row.push((
            "▶️".to_string(),
            CallbackData::PageNext(category_name.clone()).into(),
        ));
    } else {
        // Inactive button with dummy callback data
        control_row.push(("️▷".to_string(), CallbackData::Noop.into()));
    }

    button_rows.push(control_row);

    // Use pack_callback_data to create the keyboard with storage support
    let callback_storage = storage.clone().as_callback_data_storage();
    let keyboard = pack_callback_data(&callback_storage, chat_id, message_id.0, button_rows).await;

    // Add Apply and Back buttons after packing (these don't use callback data storage)
    let apply_command = if !selected_words.is_empty() {
        // Escape each word and combine with case-insensitive OR pattern with word boundaries
        let escaped_words: Vec<String> = selected_words.iter().map(|w| regex::escape(w)).collect();
        let pattern = format!(r"(?i)\b({})\b", escaped_words.join("|"));
        CommandAddFilter {
            category: Some(category_name.clone()),
            pattern: Some(pattern),
        }
        .to_command_string(true)
    } else {
        // No words selected, just put category name
        format!("{} {} ", CommandAddFilter::NAME, category_name)
    };

    // Create final keyboard with Apply and Back buttons
    let mut final_buttons = keyboard.inline_keyboard;
    final_buttons.push(vec![
        InlineKeyboardButton::switch_inline_query_current_chat("✅ Apply", apply_command),
        InlineKeyboardButton::callback("↩️ Back", CallbackData::CmdAddFilter),
    ]);

    let keyboard = InlineKeyboardMarkup::new(final_buttons);

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
    chat: Chat,
    msg_id: Option<MessageId>,
    storage: Arc<dyn StorageTrait>,
    cmd: Command,
    batch: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target = CommandReplyTarget {
        bot: bot.clone(),
        chat: chat.clone(),
        msg_id,
        batch,
        callback_data_storage: storage.clone().as_callback_data_storage(),
    };
    match cmd {
        Command::Start(start) => {
            start.run(&target, ()).await?;
        }
        Command::Help(help) => {
            help.run(&target, ()).await?;
        }
        Command::List(list) => {
            list.run(&target, storage.clone().as_expense_storage())
                .await?;
        }
        Command::Report(report) => {
            report.run(&target, storage.clone()).await?;
        }
        Command::Clear(clear) => {
            clear
                .run(&target, storage.clone().as_expense_storage())
                .await?;
        }
        Command::ClearCategories(clear_categories) => {
            clear_categories
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::AddCategory(add_category) => {
            add_category
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::Categories(categories) => {
            categories
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::AddFilter(add_filter) => {
            add_filter.run(&target, storage.clone()).await?;
        }
        Command::RemoveCategory(remove_category) => {
            remove_category
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::RemoveFilter(remove_filter) => {
            remove_filter
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::EditFilter(edit_filter) => {
            edit_filter
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::AddExpense(add_expense) => {
            add_expense
                .run(&target, storage.clone().as_expense_storage())
                .await?;
        }
        Command::AddWordsFilter(add_filter2) => {
            add_filter2.run(&target, storage.clone()).await?;
        }
    }
    Ok(())
}
