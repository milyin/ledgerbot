use teloxide::{
    prelude::*, 
    types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, ReplyMarkup},
    utils::command::BotCommands,
};

use crate::parser::format_expenses_list;
use crate::storage::{
    CategoryStorage, ExpenseStorage, add_category, add_category_filter, clear_chat_expenses, get_chat_categories,
    get_chat_expenses,
};

/// Create a persistent menu keyboard that shows on the left of the input field
pub fn create_menu_keyboard() -> ReplyMarkup {
    let keyboard = vec![
        vec![
            KeyboardButton::new("📋 /list"),
            KeyboardButton::new("🗑️ /clear"),
        ],
        vec![
            KeyboardButton::new("📂 /categories"),
            KeyboardButton::new("💡 /help"),
        ],
    ];
    ReplyMarkup::Keyboard(
        teloxide::types::KeyboardMarkup::new(keyboard)
            .resize_keyboard()
            .persistent()
    )
}

/// Bot commands
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this help")]
    Help,
    #[command(description = "start the bot")]
    Start,
    #[command(description = "show all expenses")]
    List,
    #[command(description = "clear all expenses")]
    Clear,
    #[command(description = "add expense category", parse_with = "split")]
    Category { name: String },
    #[command(description = "list all categories")]
    Categories,
    #[command(description = "add filter to category", rename = "add_filter", parse_with = "split")]
    AddFilter { category: String, pattern: String },
}

/// Display help message with inline keyboard buttons
pub async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = format!(
        "💡 **Expense Bot Help**\n\
        Version: {}\n\n\
        **How to add expenses:**\n\
        Forward messages or send text with lines in format:\n\
        `<description> <amount>`\n\n\
        **Examples:**\n\
        `Coffee 5.50`\n\
        `Lunch 12.00`\n\
        `Bus ticket 2.75`\n\n\
        **Category Examples:**\n\
        • Create a category: `/category Food`\n\
        • Add a filter: Use the Add Filter button\n\n\
        **Note:** The bot will collect your expense messages and report a summary after a few seconds of inactivity.\n\n\
        👇 **Use the buttons below:**",
        env!("CARGO_PKG_VERSION")
    );

    // Create inline keyboard with buttons for each command
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📋 List Expenses", "cmd_list"),
            InlineKeyboardButton::callback("🗑️ Clear Expenses", "cmd_clear"),
        ],
        vec![
            InlineKeyboardButton::switch_inline_query_current_chat("➕ Category", "/category "),
            InlineKeyboardButton::callback("❌ Remove Category", "cmd_remove_category"),
            InlineKeyboardButton::callback("📂 Categories", "cmd_categories"),
        ],
        vec![
            InlineKeyboardButton::callback("🔧 Add Filter", "cmd_add_filter"),
            InlineKeyboardButton::callback("🗑️ Remove Filter", "cmd_remove_filter"),
        ],
    ]);

    // Send message with both inline keyboard (for buttons in message) and reply keyboard (menu button)
    bot.send_message(msg.chat.id, help_text)
        .reply_markup(keyboard)
        .await?;
    
    // Send a follow-up message to set the persistent reply keyboard menu
    bot.send_message(msg.chat.id, "Menu buttons are now available ⬇️")
        .reply_markup(create_menu_keyboard())
        .await?;
    
    Ok(())
}

/// List all expenses
pub async fn list_command(
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

/// Add a category (name only)
pub async fn category_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    name: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    add_category(&category_storage, chat_id, name.clone()).await;
    bot.send_message(
        chat_id,
        format!(
            "✅ Category '{}' created. Use the Add Filter button to add regex patterns.",
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
    let categories = get_chat_categories(&category_storage, chat_id).await;

    // Check if category exists
    if !categories.contains_key(&category) {
        bot.send_message(
            chat_id,
            format!("❌ Category '{}' does not exist. Create it first with /category {}", category, category),
        )
        .await?;
        return Ok(());
    }

    // Validate regex pattern
    match regex::Regex::new(&pattern) {
        Ok(_) => {
            add_category_filter(&category_storage, chat_id, category.clone(), pattern.clone()).await;
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
            result.push_str(&format!("/category {}\n", name));

            // Then assign patterns if they exist
            for pattern in patterns {
                result.push_str(&format!("/add_filter {} {}\n", name, pattern));
            }
        }
        bot.send_message(chat_id, result).await?;
    }

    Ok(())
}

/// Show category removal interface
pub async fn remove_category_menu(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.send_message(chat_id, "No categories to remove.")
            .await?;
    } else {
        let text = "❌ **Select category to remove:**";

        // Create buttons for each category
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::callback(
                    format!("🚫 {}", name),
                    format!("remove_cat:{}", name),
                )]
            })
            .collect();

        // Add a back button
        buttons.push(vec![InlineKeyboardButton::callback(
            "⬅️ Back to Menu",
            "cmd_back_to_help",
        )]);

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.send_message(chat_id, text)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show add filter interface - first show categories
pub async fn add_filter_menu(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.send_message(chat_id, "No categories available. Create a category first with /category <name>")
            .await?;
    } else {
        let text = "� **Select category to add filter:**";

        // Create buttons for each category
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    format!("🔧 {}", name),
                    format!("/add_filter {} ", name),
                )]
            })
            .collect();

        // Add a back button
        buttons.push(vec![InlineKeyboardButton::callback(
            "⬅️ Back to Menu",
            "cmd_back_to_help",
        )]);

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.send_message(chat_id, text)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show remove filter interface - first show categories
pub async fn remove_filter_menu(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if categories.is_empty() {
        bot.send_message(chat_id, "No categories available.")
            .await?;
    } else {
        let text = "�️ **Select category to remove filter:**";

        // Create buttons for each category that has filters
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = categories
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
            bot.send_message(chat_id, "No filters defined in any category.")
                .await?;
            return Ok(());
        }

        // Add a back button
        buttons.push(vec![InlineKeyboardButton::callback(
            "⬅️ Back to Menu",
            "cmd_back_to_help",
        )]);

        let keyboard = InlineKeyboardMarkup::new(buttons);

        bot.send_message(chat_id, text)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

/// Show filters for a specific category for removal
pub async fn show_category_filters_for_removal(
    bot: Bot,
    chat_id: ChatId,
    category_storage: CategoryStorage,
    category_name: String,
) -> ResponseResult<()> {
    let categories = get_chat_categories(&category_storage, chat_id).await;

    if let Some(patterns) = categories.get(&category_name) {
        if patterns.is_empty() {
            bot.send_message(chat_id, format!("No filters in category '{}'.", category_name))
                .await?;
        } else {
            let text = format!("�️ **Select filter to remove from '{}':**", category_name);

            // Create buttons for each filter
            let mut buttons: Vec<Vec<InlineKeyboardButton>> = patterns
                .iter()
                .map(|pattern| {
                    vec![InlineKeyboardButton::callback(
                        pattern.clone(),
                        format!("remove_filter:{}:{}", category_name, pattern),
                    )]
                })
                .collect();

            // Add a back button
            buttons.push(vec![InlineKeyboardButton::callback(
                "↩️ Back",
                "cmd_remove_filter",
            )]);

            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.send_message(chat_id, text)
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
        Command::Help | Command::Start => help_command(bot, msg).await,
        Command::List => list_command(bot, msg, storage, category_storage).await,
        Command::Clear => clear_command(bot, msg, storage).await,
        Command::Category { name } => category_command(bot, msg, category_storage, name).await,
        Command::Categories => categories_command(bot, msg, category_storage).await,
        Command::AddFilter { category, pattern } => add_filter_command(bot, msg, category_storage, category, pattern).await,
    }
}
