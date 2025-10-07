use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, MessageId, ReplyMarkup},
    utils::command::{BotCommands, ParseError},
};

use crate::parser::{extract_words, format_expenses_chronological, format_expenses_list};
use crate::storage::{
    CategoryStorage, ExpenseStorage, FilterSelectionStorage, add_category, add_category_filter,
    clear_chat_expenses, get_chat_categories, get_chat_expenses, get_filter_selection,
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

/// Custom parser for three optional string parameters (date, description, amount)
fn parse_three_optional_strings(
    s: String,
) -> Result<(Option<String>, Option<String>, Option<String>), ParseError> {
    // Take only the first line to prevent multi-line capture
    let first_line = s.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return Ok((None, None, None));
    }

    let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
    match parts.as_slice() {
        [first] => Ok((Some(first.to_string()), None, None)),
        [first, second] => Ok((Some(first.to_string()), Some(second.to_string()), None)),
        [first, second, third] => Ok((
            Some(first.to_string()),
            Some(second.to_string()),
            Some(third.to_string()),
        )),
        _ => Ok((None, None, None)),
    }
}

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
        parse_with = parse_three_optional_strings
    )]
    Expense {
        date: Option<String>,
        description: Option<String>,
        amount: Option<String>,
    },
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
    bot.send_message(msg.chat.id, help_text).await?;
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

/// Handle expense command with date, description, and amount
pub async fn expense_command(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
    date: Option<String>,
    description: Option<String>,
    amount: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Get message timestamp for default date
    let message_timestamp = msg.forward_date().unwrap_or(msg.date).timestamp();

    // Validate and parse parameters
    match (description, amount) {
        (Some(desc), Some(amt_str)) => {
            // Parse amount
            match amt_str.parse::<f64>() {
                Ok(amount_val) => {
                    // Determine timestamp
                    let timestamp = if let Some(ref date_str) = date {
                        // Try to parse the date
                        crate::parser::parse_date_to_timestamp(date_str)
                            .unwrap_or(message_timestamp)
                    } else {
                        message_timestamp
                    };

                    // Store the expense
                    crate::storage::add_expenses(
                        &storage,
                        chat_id,
                        vec![(desc.clone(), amount_val, timestamp)],
                    )
                    .await;

                    // Format date for display
                    let date_display = if let Some(d) = date {
                        d
                    } else {
                        use chrono::{DateTime, Utc};
                        let dt: DateTime<Utc> =
                            DateTime::from_timestamp(timestamp, 0).unwrap_or_default();
                        dt.format("%Y-%m-%d").to_string()
                    };

                    bot.send_message(
                        chat_id,
                        format!("✅ Expense added: {} {} {}", date_display, desc, amount_val),
                    )
                    .await?;
                }
                Err(_) => {
                    bot.send_message(
                        chat_id,
                        format!(
                            "❌ Invalid amount: '{}'. Please provide a valid number.",
                            amt_str
                        ),
                    )
                    .await?;
                }
            }
        }
        _ => {
            bot.send_message(
                chat_id,
                "❌ Missing parameters. Usage: /expense <date> <description> <amount>\nOr: /expense <description> <amount> (uses current date)"
            ).await?;
        }
    }

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

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat("➕ Add Category", "/add_category "),
    ]]);

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
    name: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if name is provided
    match name {
        None => {
            // Show the add category menu instead
            let sent_msg = bot.send_message(chat_id, "➕ Add Category").await?;
            add_category_menu(bot, chat_id, sent_msg.id).await?;
        }
        Some(name) => match add_category(&category_storage, chat_id, name.clone()).await {
            Ok(()) => {
                bot.send_message(
                    chat_id,
                    format!(
                        "✅ Category '{}' created. Use /add_filter to add regex patterns.",
                        name
                    ),
                )
                .await?;
            }
            Err(err_msg) => {
                bot.send_message(chat_id, format!("ℹ️ {}", err_msg)).await?;
            }
        },
    }

    Ok(())
}

/// Add a filter to a category
pub async fn add_filter_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    category: Option<String>,
    pattern: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, pattern) {
        (Some(category), Some(pattern)) => {
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
        }
        (None, None) => {
            // Show the add filter menu instead
            let sent_msg = bot.send_message(chat_id, "🔧 Add Filter").await?;
            add_filter_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        }
        (Some(category), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing pattern. Usage: /add_filter {} <pattern>",
                    category
                ),
            )
            .await?;
        }
        (None, Some(_)) => {
            bot.send_message(
                chat_id,
                "❌ Missing category. Usage: /add_filter <category> <pattern>",
            )
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
    name: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match name {
        None => {
            // Show the remove category menu instead
            let sent_msg = bot.send_message(chat_id, "❌ Remove Category").await?;
            remove_category_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        }
        Some(name) => {
            let categories = get_chat_categories(&category_storage, chat_id).await;

            // Check if category exists
            if !categories.contains_key(&name) {
                bot.send_message(chat_id, format!("❌ Category '{}' does not exist.", name))
                    .await?;
                return Ok(());
            }

            // Remove the category
            crate::storage::remove_category(&category_storage, chat_id, &name).await;
            bot.send_message(chat_id, format!("✅ Category '{}' removed.", name))
                .await?;
        }
    }

    Ok(())
}

/// Remove a filter from a category
pub async fn remove_filter_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    category: Option<String>,
    pattern: Option<String>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match (category, pattern) {
        (Some(category), Some(pattern)) => {
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
            if let Some(patterns) = categories.get(&category)
                && !patterns.contains(&pattern)
            {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ Filter '{}' not found in category '{}'.",
                        pattern, category
                    ),
                )
                .await?;
                return Ok(());
            }

            // Remove the filter
            crate::storage::remove_category_filter(&category_storage, chat_id, &category, &pattern)
                .await;
            bot.send_message(
                chat_id,
                format!(
                    "✅ Filter '{}' removed from category '{}'.",
                    pattern, category
                ),
            )
            .await?;
        }
        (None, None) => {
            // Show the remove filter menu instead
            let sent_msg = bot.send_message(chat_id, "🗑️ Remove Filter").await?;
            remove_filter_menu(bot, chat_id, sent_msg.id, category_storage).await?;
        }
        (Some(category), None) => {
            bot.send_message(
                chat_id,
                format!(
                    "❌ Missing pattern. Usage: /remove_filter {} <pattern>",
                    category
                ),
            )
            .await?;
        }
        (None, Some(_)) => {
            bot.send_message(
                chat_id,
                "❌ Missing category. Usage: /remove_filter <category> <pattern>",
            )
            .await?;
        }
    }

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
            let text = format!(
                "�️ **Select filter to remove from '{}':**\n\nClick a button to place the command in your input box.",
                category_name
            );

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
