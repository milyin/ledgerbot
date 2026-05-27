use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::{prelude::ResponseResult, types::ChatId};

use crate::{impl_command_execute_0, impl_command_io, storages::Stores, ui::CommandContext};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandUnfollow;

impl_command_io!(CommandUnfollow, "unfollow", []);
impl_command_execute_0!(CommandUnfollow, Arc<Stores>);

impl CommandUnfollow {
    async fn run0(&self, target: &CommandContext, storage: Arc<Stores>) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let storage_ = storage.storage(chat_id);
        let variable_storage = storage_.variables();
        let followed_chat: Option<ChatId> = variable_storage.get().await;

        match followed_chat {
            Some(chat_id) => {
                variable_storage.remove::<ChatId>().await;
                target
                    .send_markdown_message(markdown_format!(
                        "✅ Stopped following expenses from chat `{}`\\.",
                        chat_id.0
                    ))
                    .await?;
            }
            None => {
                target
                    .send_markdown_message(markdown_format!("ℹ️ You are not following any chat\\."))
                    .await?;
            }
        }

        Ok(())
    }
}

impl From<CommandUnfollow> for crate::commands::Command {
    fn from(cmd: CommandUnfollow) -> Self {
        crate::commands::Command::Unfollow(cmd)
    }
}
