use teloxide::{
    prelude::*, types::InlineKeyboardButton, types::InlineKeyboardMarkup,
    utils::command::BotCommands,
};

use crate::parser::format_expenses_list;
use crate::storage::{
    CategoryStorage, ExpenseStorage, add_category, clear_chat_expenses, get_chat_categories,
    get_chat_expenses,
};

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
    #[command(
        description = "assign regex pattern to existing category",
        parse_with = "split"
    )]
    Assign { name: String, pattern: String },
    #[command(description = "list all categories")]
    Categories,
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
        • Assign pattern: `/assign Food (coffee|lunch|dinner)`\n\n\
        **Note:** The bot will collect your expense messages and report a summary after a few seconds of inactivity.\n\n\
        👇 **Use the buttons below:**",
        env!("CARGO_PKG_VERSION")
    );

    // Create inline keyboard with buttons for each command
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📋 List", "cmd_list"),
            InlineKeyboardButton::callback("🗑️ Clear", "cmd_clear"),
        ],
        vec![
            InlineKeyboardButton::switch_inline_query_current_chat("➕ Category", "/category "),
            InlineKeyboardButton::switch_inline_query_current_chat("🔗 Assign", "/assign "),
        ],
        vec![
            InlineKeyboardButton::callback("📁 Categories", "cmd_categories"),
            InlineKeyboardButton::callback("🗑️ Remove Category", "cmd_remove_category"),
        ],
    ]);

    bot.send_message(msg.chat.id, help_text)
        .reply_markup(keyboard)
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

    add_category(&category_storage, chat_id, name.clone(), String::new()).await;
    bot.send_message(
        chat_id,
        format!(
            "✅ Category '{}' created. Use /assign to add a regex pattern.",
            name
        ),
    )
    .await?;

    Ok(())
}

/// Assign a regex pattern to an existing category
pub async fn assign_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    name: String,
    pattern: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // Check if category exists
    let categories = get_chat_categories(&category_storage, chat_id).await;
    if !categories.contains_key(&name) {
        bot.send_message(
            chat_id,
            format!(
                "❌ Category '{}' does not exist. Use /category to create it first.",
                name
            ),
        )
        .await?;
        return Ok(());
    }

    // Validate regex pattern
    match regex::Regex::new(&pattern) {
        Ok(_) => {
            add_category(&category_storage, chat_id, name.clone(), pattern.clone()).await;
            bot.send_message(
                chat_id,
                format!("✅ Category '{}' assigned pattern: {}", name, pattern),
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

        for (name, pattern) in sorted_categories {
            // First create the category
            result.push_str(&format!("/category {}\n", name));

            // Then assign pattern if it exists
            if !pattern.is_empty() {
                result.push_str(&format!("/assign {} {}\n", name, pattern));
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
        let text = "🗑️ **Select category to remove:**";

        // Create buttons for each category
        let mut buttons: Vec<Vec<InlineKeyboardButton>> = categories
            .keys()
            .map(|name| {
                vec![InlineKeyboardButton::callback(
                    format!("❌ {}", name),
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
        Command::Assign { name, pattern } => {
            assign_command(bot, msg, category_storage, name, pattern).await
        }
        Command::Categories => categories_command(bot, msg, category_storage).await,
    }
}
