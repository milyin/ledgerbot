use teloxide::{prelude::*, utils::command::BotCommands};

use crate::parser::format_expenses_list;
use crate::storage::{add_category, clear_chat_expenses, get_chat_categories, get_chat_expenses, CategoryStorage, ExpenseStorage};

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
    #[command(description = "add expense category with regex pattern", parse_with = "split")]
    Category { name: String, pattern: String },
    #[command(description = "list all categories")]
    Categories,
}

/// Display help message with auto-generated command list
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
        **Commands:**\n\
        {}\n\n\
        **Category Examples:**\n\
        `/category Food (coffee|lunch|dinner)` - Match food expenses\n\
        `/category Transport (bus|taxi|uber)` - Match transport expenses\n\
        `/categories` - Show all your categories\n\n\
        **Note:** The bot will collect your expense messages and report a summary after a few seconds of inactivity.",
        env!("CARGO_PKG_VERSION"),
        Command::descriptions()
    );

    bot.send_message(msg.chat.id, help_text).await?;
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

/// Add a category with regex pattern
pub async fn category_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
    name: String,
    pattern: String,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    
    // Validate regex pattern
    match regex::Regex::new(&pattern) {
        Ok(_) => {
            add_category(&category_storage, chat_id, name.clone(), pattern.clone()).await;
            bot.send_message(
                chat_id,
                format!("✅ Category '{}' added with pattern: {}", name, pattern),
            )
            .await?;
        }
        Err(e) => {
            bot.send_message(
                chat_id,
                format!("❌ Invalid regex pattern: {}", e),
            )
            .await?;
        }
    }
    
    Ok(())
}

/// List all categories
pub async fn categories_command(
    bot: Bot,
    msg: Message,
    category_storage: CategoryStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let categories = get_chat_categories(&category_storage, chat_id).await;
    
    if categories.is_empty() {
        bot.send_message(chat_id, "No categories defined yet.").await?;
    } else {
        let mut result = "📁 **Categories:**\n\n".to_string();
        for (name, pattern) in categories.iter() {
            result.push_str(&format!("• **{}**: `{}`\n", name, pattern));
        }
        bot.send_message(chat_id, result).await?;
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
        Command::Category { name, pattern } => {
            category_command(bot, msg, category_storage, name, pattern).await
        }
        Command::Categories => categories_command(bot, msg, category_storage).await,
    }
}
