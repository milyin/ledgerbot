use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, MessageId, ReplyMarkup},
    utils::command::BotCommands,
};

use crate::parser::{extract_words, format_expenses_chronological, format_expenses_list};
use crate::storage::{
    CategoryStorage, ExpenseStorage, FilterSelectionStorage, add_category, add_category_filter,
    clear_chat_expenses, get_chat_categories, get_chat_expenses, get_filter_selection,
};

/// Create a persistent menu keyboard that shows on the left of the input field
pub fn create_menu_keyboard() -> ReplyMarkup {
    let keyboard = vec![vec![
        KeyboardButton::new("💡 /help"),
        KeyboardButton::new("🗒️ /list"),
        KeyboardButton::new("🗂 /categories"),
        KeyboardButton::new("📋 /report"),
    ]];
    ReplyMarkup::Keyboard(
        teloxide::types::KeyboardMarkup::new(keyboard)
            .resize_keyboard()
            .persistent(),
    )
}

/// Bot commands
#[derive(BotCommands, Clone)]
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
    #[command(description = "add expense category", rename = "add_category", parse_with = "split")]
    AddCategory { name: String },
    #[command(
        description = "add filter to category",
        rename = "add_filter",
        parse_with = "split"
    )]
    AddFilter { category: String, pattern: String },
    #[command(
        description = "remove expense category",
        rename = "remove_category",
        parse_with = "split"
    )]
    RemoveCategory { name: String },
    #[command(
        description = "remove filter from category",
        rename = "remove_filter",
        parse_with = "split"
    )]
    RemoveFilter { category: String, pattern: String },
}

/// Display help message with inline keyboard buttons
pub async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = format!(
        "To add expenses forward messages or send text with lines in format:\n\
        `[<yyyy-mm-dd>] <description> <amount>\n\n\
        {commands}",
        commands = Command::descriptions()
    );

    // Send message with both inline keyboard (for buttons in message) and reply keyboard (menu button)
    bot.send_message(msg.chat.id, help_text)
        .await?;
    Ok(())
}

pub async fn start_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    // Send a follow-up message to set the persistent reply keyboard menu
    bot.send_message(
        msg.chat.id,
        format!(
            "Expense Bot v.{}\nMenu buttons are available ⬇️",
            env!("CARGO_PKG_VERSION"),
        ),
    )
    .reply_markup(create_menu_keyboard())
    .await?;

    help_command(bot, msg).await?;

    Ok(())
}
/// List all expenses chronologically without category grouping
pub async fn list_command(bot: Bot, msg: Message, storage: ExpenseStorage) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = get_chat_expenses(&storage, chat_id).await;
    let expenses_list = format_expenses_chronological(&chat_expenses);

    bot.send_message(chat_id, expenses_list).await?;
    Ok(())
}

/// Report all expenses grouped by categories
pub async fn report_command(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = get_chat_expenses(&storage, chat_id).await;
    let chat_categories = get_chat_categories(&category_storage, chat_id).await;
    let expenses_list = format_expenses_list(&chat_expenses, &chat_categories);

    bot.send_message(chat_id, expenses_list).await?;
    Ok(())
}

/// Clear all expenses
pub async fn clear_command(bot: Bot, msg: Message, storage: ExpenseStorage) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    clear_chat_expenses(&storage, chat_id).await;

    bot.send_message(chat_id, "🗑️ All expenses cleared!")
        .await?;
    Ok(())
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

/// Show add category menu
pub async fn add_category_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
) -> ResponseResult<()> {
    let text = "➕ **Add a new category:**\n\nClick the button below and type the category name.";

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::switch_inline_query_current_chat(
            "➕ Add Category",
            "/add_category ",
        )],
    ]);

    bot.edit_message_text(chat_id, message_id, text).await?;
    bot.edit_message_reply_markup(chat_id, message_id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Add a category (name only)
pub async fn category_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    name: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if name is empty or just whitespace
    if name.trim().is_empty() {
        // Show the add category menu instead
        let sent_msg = bot.send_message(chat_id, "➕ Add Category").await?;
        add_category_menu(bot, chat_id, sent_msg.id).await?;
        return Ok(());
    }

    add_category(&category_storage, chat_id, name.clone()).await;
    bot.send_message(
        chat_id,
        format!(
            "✅ Category '{}' created. Use /add_filter to add regex patterns.",
            name
        ),
    )
    .await?;

    Ok(())
}

/// Add a filter to a category
pub async fn add_filter_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    category: String,
    pattern: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if category or pattern is empty or just whitespace
    if category.trim().is_empty() || pattern.trim().is_empty() {
        // Show the add filter menu instead
        let sent_msg = bot.send_message(chat_id, "🔧 Add Filter").await?;
        add_filter_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        return Ok(());
    }

    let categories = get_chat_categories(&category_storage, chat_id).await;

    // Check if category exists
    if !categories.contains_key(&category) {
        bot.send_message(
            chat_id,
            format!(
                "❌ Category '{}' does not exist. Create it first with /add_category {}",
                category, category
            ),
        )
        .await?;
        return Ok(());
    }

    // Treat the pattern as a regexp directly without additional wrapping
    // Validate regex pattern
    match regex::Regex::new(&pattern) {
        Ok(_) => {
            add_category_filter(
                &category_storage,
                chat_id,
                category.clone(),
                pattern.clone(),
            )
            .await;
            bot.send_message(
                chat_id,
                format!("✅ Filter '{}' added to category '{}'.", pattern, category),
            )
            .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ Invalid regex pattern: {}", e))
                .await?;
        }
    }

    Ok(())
}

/// List all categories as executable commands
pub async fn categories_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.send_message(chat_id, "No categories defined yet.")
            .await?;
    } else {
        let mut result = String::new();

        // Sort categories for consistent output
        let mut sorted_categories: Vec<_> = categories.iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

        for (name, patterns) in sorted_categories {
            // First create the category
            result.push_str(&format!("/add_category {}\n", name));

            // Then assign patterns if they exist
            for pattern in patterns {
                result.push_str(&format!("/add_filter {} {}\n", name, pattern));
            }
        }
        bot.send_message(chat_id, result).await?;
    }

    Ok(())
}

/// Remove a category
pub async fn remove_category_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    name: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if name is empty or just whitespace
    if name.trim().is_empty() {
        // Show the remove category menu instead
        let sent_msg = bot.send_message(chat_id, "❌ Remove Category").await?;
        remove_category_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        return Ok(());
    }

    let categories = get_chat_categories(&category_storage, chat_id).await;

    // Check if category exists
    if !categories.contains_key(&name) {
        bot.send_message(
            chat_id,
            format!("❌ Category '{}' does not exist.", name),
        )
        .await?;
        return Ok(());
    }

    // Remove the category
    crate::storage::remove_category(&category_storage, chat_id, &name).await;
    bot.send_message(chat_id, format!("✅ Category '{}' removed.", name))
        .await?;

    Ok(())
}

/// Remove a filter from a category
pub async fn remove_filter_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    category: String,
    pattern: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if category or pattern is empty or just whitespace
    if category.trim().is_empty() || pattern.trim().is_empty() {
        // Show the remove filter menu instead
        let sent_msg = bot.send_message(chat_id, "🗑️ Remove Filter").await?;
        remove_filter_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        return Ok(());
    }

    let categories = get_chat_categories(&category_storage, chat_id).await;

    // Check if category exists
    if !categories.contains_key(&category) {
        bot.send_message(
            chat_id,
            format!("❌ Category '{}' does not exist.", category),
        )
        .await?;
        return Ok(());
    }

    // Check if filter exists in the category
    if let Some(patterns) = categories.get(&category) {
        if !patterns.contains(&pattern) {
            bot.send_message(
                chat_id,
                format!("❌ Filter '{}' not found in category '{}'.", pattern, category),
            )
            .await?;
            return Ok(());
        }
    }

    // Remove the filter
    crate::storage::remove_category_filter(&category_storage, chat_id, &category, &pattern).await;
    bot.send_message(
        chat_id,
        format!("✅ Filter '{}' removed from category '{}'.", pattern, category),
    )
    .await?;

    Ok(())
}

/// Show category removal interface
pub async fn remove_category_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(chat_id, message_id, "No categories to remove.")
            .await?;
    } else {
        let text = "❌ **Select category to remove:**\n\nClick a button to place the command in your input box.";

        // Create buttons for each category using switch_inline_query_current_chat
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    format!("🚫 {}", name),
                    format!("/remove_category {}", name),
                )]
            })
            .collect();

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.edit_message_text(chat_id, message_id, text).await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show add filter interface - first show categories
pub async fn add_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(
            chat_id,
            message_id,
            "No categories available. Create a category first with /add_category <name>",
        )
        .await?;
    } else {
        let text = "� **Select category to add filter:**";

        // Create buttons for each category
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::callback(
                    format!("🔧 {}", name),
                    format!("add_filter_cat:{}", name),
                )]
            })
            .collect();

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.edit_message_text(chat_id, message_id, text).await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

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
    category_name: String,
) -> ResponseResult<()> {
    let expenses = get_chat_expenses(&storage, chat_id).await;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    // Get currently selected words from storage
    let selected_words =
        get_filter_selection(&filter_selection_storage, chat_id, &category_name).await;

    // Extract words from uncategorized expenses
    let words = extract_words(&expenses, &categories);

    // Build selected words display
    let selected_display = if selected_words.is_empty() {
        "(none selected)".to_string()
    } else {
        selected_words.join(" | ")
    };

    let text = format!(
        "💡 **Select word(s) for filter '{}':**\n\nSelected: {}\n\nClick words to add/remove them. When done, click 'Apply Filter'.",
        category_name, selected_display
    );

    let mut buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    // Add buttons for each suggested word (limit to 20 most common ones, 4 per row)
    let mut row: Vec<InlineKeyboardButton> = Vec::new();
    for word in words.iter().take(20) {
        // Check if this word is selected
        let is_selected = selected_words.contains(word);
        let label = if is_selected {
            format!("✓ {}", word)
        } else {
            word.clone()
        };

        // Use short callback data without encoding state
        row.push(InlineKeyboardButton::callback(
            label,
            format!("toggle_word:{}:{}", category_name, word),
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

    // Add apply button if words are selected
    if !selected_words.is_empty() {
        buttons.push(vec![InlineKeyboardButton::callback(
            "✅ Apply Filter",
            format!("apply_words:{}", category_name),
        )]);
    }

    // Add custom filter button
    buttons.push(vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            "✏️ Custom Filter",
            format!("/add_filter {} ", category_name),
        ),
    ]);

    // Add a back button
    buttons.push(vec![InlineKeyboardButton::callback(
        "↩️ Back",
        "cmd_add_filter",
    )]);

    let keyboard = InlineKeyboardMarkup::new(buttons);

    bot.edit_message_text(chat_id, message_id, text).await?;
    bot.edit_message_reply_markup(chat_id, message_id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Show remove filter interface - first show categories
pub async fn remove_filter_menu(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.edit_message_text(chat_id, message_id, "No categories available.")
            .await?;
    } else {
        let text = "�️ **Select category to remove filter:**\n\nClick a button to see filters for that category.";

        // Create buttons for each category that has filters
        let buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .iter()
            .filter(|(_, patterns)| !patterns.is_empty())
            .map(|(name, _)| {
                vec![InlineKeyboardButton::callback(
                    format!("�️ {}", name),
                    format!("remove_filter_cat:{}", name),
                )]
            })
            .collect();

        if buttons.is_empty() {
            bot.edit_message_text(chat_id, message_id, "No filters defined in any category.")
                .await?;
            return Ok(());
        }

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.edit_message_text(chat_id, message_id, text).await?;
        bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show filters for a specific category for removal
pub async fn show_category_filters_for_removal(
    bot: Bot,
    chat_id: ChatId,
    message_id: MessageId,
    category_storage: CategoryStorage,
    category_name: String,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if let Some(patterns) = categories.get(&category_name) {
        if patterns.is_empty() {
            bot.edit_message_text(
                chat_id,
                message_id,
                format!("No filters in category '{}'.", category_name),
            )
            .await?;
        } else {
            let text = format!("�️ **Select filter to remove from '{}':**\n\nClick a button to place the command in your input box.", category_name);

            // Create buttons for each filter using switch_inline_query_current_chat
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = patterns
                .iter()
                .map(|pattern| {
                    vec![InlineKeyboardButton::switch_inline_query_current_chat(
                        pattern.clone(),
                        format!("/remove_filter {} {}", category_name, pattern),
                    )]
                })
                .collect();

            // Add a back button
            buttons.push(vec![InlineKeyboardButton::callback(
                "↩️ Back",
                "cmd_remove_filter",
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.edit_message_text(chat_id, message_id, text).await?;
            bot.edit_message_reply_markup(chat_id, message_id)
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}

/// Unified command handler
pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    storage: ExpenseStorage,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => start_command(bot, msg).await,
        Command::Help => help_command(bot, msg).await,
        Command::List => list_command(bot, msg, storage).await,
        Command::Report => report_command(bot, msg, storage, category_storage.clone()).await,
        Command::Clear => clear_command(bot, msg, storage).await,
        Command::AddCategory { name } => category_command(bot, msg, category_storage.clone(), name).await,
        Command::Categories => categories_command(bot, msg, category_storage.clone()).await,
        Command::ClearCategories => clear_categories_command(bot, msg, category_storage.clone()).await,
        Command::AddFilter { category, pattern } => {
            add_filter_command(bot, msg, category_storage.clone(), category, pattern).await
        }
        Command::RemoveCategory { name } => {
            remove_category_command(bot, msg, category_storage.clone(), name).await
        }
        Command::RemoveFilter { category, pattern } => {
            remove_filter_command(bot, msg, category_storage, category, pattern).await
        }
    }
}
