use teloxide::{prelude::ResponseResult, utils::command::BotCommands};
use yoroolbot::markdown_format;

use super::Command;
use crate::commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandHelp;

impl CommandTrait for CommandHelp {
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

    const NAME: &'static str = "help";
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
        CommandHelp
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _context: Self::Context,
    ) -> ResponseResult<()> {
        target
            .send_markdown_message(markdown_format!(
                "To add expenses forward messages or send text with lines in format:\n\
            `\\[\\<yyyy\\-mm\\-dd\\>\\] \\<description\\> \\<amount\\>`\n\n\
            {}",
                Command::descriptions().to_string()
            ))
            .await?;
        Ok(())
    }
}

impl From<CommandHelp> for crate::commands::Command {
    fn from(cmd: CommandHelp) -> Self {
        crate::commands::Command::Help(cmd)
    }
}
