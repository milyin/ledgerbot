use teloxide::{prelude::*, utils::command::BotCommands};

use crate::parser::format_expenses_list;
use crate::storage::{clear_chat_expenses, get_chat_expenses, ExpenseStorage};

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
}

/// Display help message with auto-generated command list
pub async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = format!(
        "💡 **Expense Bot Help**\n\n\
        **How to add expenses:**\n\
        Forward messages or send text with lines in format:\n\
        `<description> <amount>`\n\n\
        **Examples:**\n\
        `Coffee 5.50`\n\
        `Lunch 12.00`\n\
        `Bus ticket 2.75`\n\n\
        **Commands:**\n\
        {}\n\n\
        **Note:** The bot will collect your expense messages and report a summary after a few seconds of inactivity.",
        Command::descriptions()
    );

    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}

/// List all expenses
pub async fn list_command(bot: Bot, msg: Message, storage: ExpenseStorage) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let chat_expenses = get_chat_expenses(&storage, chat_id).await;
    let expenses_list = format_expenses_list(&chat_expenses);

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

/// Unified command handler
pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    storage: ExpenseStorage,
) -> ResponseResult<()> {
    match cmd {
        Command::Help | Command::Start => help_command(bot, msg).await,
        Command::List => list_command(bot, msg, storage).await,
        Command::Clear => clear_command(bot, msg, storage).await,
    }
}
