use teloxide::{
    payloads::SendMessageSetters,
    prelude::ResponseResult,
    types::{KeyboardButton, ReplyMarkup},
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format};

use crate::commands::{
    command_help::CommandHelp,
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandStart;

impl CommandTrait for CommandStart {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = ();

    const NAME: &'static str = "start";
    const PLACEHOLDERS: &[&'static str] = &[];

    fn from_arguments(
        _: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandStart
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _context: Self::Context,
    ) -> ResponseResult<()> {
        // Send a follow-up message to set the persistent reply keyboard menu
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

        // Use CommandHelp to display help
        CommandHelp
            .run(
                target,
                (),
            )
            .await?;

        Ok(())
    }
}

impl From<CommandStart> for crate::commands::Command {
    fn from(cmd: CommandStart) -> Self {
        crate::commands::Command::Start(cmd)
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
