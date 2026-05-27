use serde::{Deserialize, Serialize};
use telluride::{markdown::MarkdownStringMessage, markdown_format};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::ResponseResult,
    types::{KeyboardButton, ReplyMarkup},
};

use crate::{
    commands::command_help::CommandHelp, impl_command_execute_0, impl_command_io,
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandStart;

impl_command_io!(CommandStart, "start", []);
impl_command_execute_0!(CommandStart, ());

impl CommandStart {
    async fn run0(&self, target: &CommandContext, _context: ()) -> ResponseResult<()> {
        target
            .bot
            .send_markdown_message(
                target.chat.id,
                markdown_format!(
                    "🤖 *Expense Bot v{}*\nMenu buttons are available",
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .reply_markup(create_menu_keyboard())
            .await?;

        CommandHelp.execute(target, ()).await?;
        Ok(())
    }
}

impl From<CommandStart> for crate::commands::Command {
    fn from(cmd: CommandStart) -> Self {
        crate::commands::Command::Start(cmd)
    }
}

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
