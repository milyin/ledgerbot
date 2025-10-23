use teloxide::{
    prelude::*,
    types::{KeyboardButton, Message, ReplyMarkup},
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format};

use crate::commands::{
    command_help::CommandHelp,
    command_trait::{CommandReplyTarget, CommandTrait},
};

pub async fn start_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    // Send a follow-up message to set the persistent reply keyboard menu
    bot.send_markdown_message(
        msg.chat.id,
        markdown_format!(
            "🤖 *Expense Bot v{}*\nMenu buttons are available",
            env!("CARGO_PKG_VERSION")
        ),
    )
    .reply_markup(create_menu_keyboard())
    .await?;

    // Use CommandHelp to display help
    CommandHelp
        .run(
            &CommandReplyTarget {
                bot,
                chat: msg.chat,
                msg_id: None,
            },
            (),
        )
        .await?;

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
