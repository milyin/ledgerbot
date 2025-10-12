use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{KeyboardButton, Message, ReplyMarkup},
    utils::command::BotCommands,
};

use crate::send_message_markdown;
use super::Command;

/// Display help message with inline keyboard buttons
pub async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = format!(
        "To add expenses forward messages or send text with lines in format:\n\
        <code>[&lt;yyyy-mm-dd&gt;] &lt;description&gt; &lt;amount&gt;</code>\n\n\
        {commands}",
        commands = Command::descriptions()
    );

    // Send message with both inline keyboard (for buttons in message) and reply keyboard (menu button)
    bot.send_message(msg.chat.id, help_text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn start_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    // Send a follow-up message to set the persistent reply keyboard menu
    send_message_markdown!(
        bot,
        msg.chat.id,
        "🤖 *Expense Bot v{}*\nMenu buttons are available",
        env!("CARGO_PKG_VERSION")
    )
    .reply_markup(create_menu_keyboard())
    .await?;

    help_command(bot, msg).await?;

    Ok(())
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
