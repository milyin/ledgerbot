use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::{prelude::ResponseResult, utils::command::BotCommands};

use crate::{commands::Command, impl_command_execute_0, impl_command_io, ui::CommandContext};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandHelp;

impl_command_io!(CommandHelp, "help", []);
impl_command_execute_0!(CommandHelp, ());

impl CommandHelp {
    async fn run0(&self, target: &CommandContext, _context: ()) -> ResponseResult<()> {
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
