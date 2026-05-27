use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::{prelude::ResponseResult, types::ChatId};

use crate::{
    commands::{command_unfollow::CommandUnfollow, follow_helper},
    impl_command_execute_1, impl_command_io,
    storages::Stores,
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandFollow {
    pub chat_id: Option<i64>,
}

impl CommandFollow {
    pub fn new(chat_id: ChatId) -> Self {
        CommandFollow {
            chat_id: Some(chat_id.0),
        }
    }
}

impl From<CommandFollow> for crate::commands::Command {
    fn from(cmd: CommandFollow) -> Self {
        crate::commands::Command::Follow(cmd)
    }
}

impl_command_io!(CommandFollow, "follow", ["<chat_id>"], chat_id: i64);
impl_command_execute_1!(CommandFollow, Arc<Stores>, chat_id);

impl CommandFollow {
    async fn run0(&self, target: &CommandContext, storage: Arc<Stores>) -> ResponseResult<()> {
        let storage_ = storage.storage(target.chat.id);
        let variable_storage = storage_.variables();
        let followed_chat: Option<ChatId> = variable_storage.get().await;

        let status_message = match followed_chat {
            Some(chat_id) => {
                match follow_helper::validate_and_get_follow_access(target, &storage).await {
                    Ok(_) => markdown_format!(
                        "👁️ **Currently following:** chat `{}`\n\n\
                     To change, use `{}`\n\
                     To stop following, use `{}`",
                        chat_id.0,
                        CommandFollow::default().to_command_string(true),
                        CommandUnfollow.to_command_string(true)
                    ),
                    Err(warning_msg) => {
                        warning_msg
                            + markdown_format!(
                                "\n\nℹ️ Usage: `{}`\n\n\
                             Follow expenses from another chat that has shared access to you\\.",
                                CommandFollow::default().to_command_string(true)
                            )
                    }
                }
            }
            None => markdown_format!(
                "ℹ️ Not currently following any chat\\.\n\n\
                 Usage: `{}`\n\n\
                 Follow expenses from another chat that has shared access to you\\.",
                CommandFollow::default().to_command_string(true)
            ),
        };

        target.send_markdown_message(status_message).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<Stores>,
        target_chat_id: &i64,
    ) -> ResponseResult<()> {
        let target_chat_id = ChatId(*target_chat_id);

        if let Err(error_msg) =
            follow_helper::validate_follow_access(target, &storage, target_chat_id).await
        {
            target.send_markdown_message(error_msg).await?;
            return Ok(());
        }

        let storage_ = storage.storage(target.chat.id);
        let variable_storage = storage_.variables();
        variable_storage.set(target_chat_id).await;

        target
            .send_markdown_message(markdown_format!(
                "✅ Now following expenses from chat `{}`\\.\n\nTo stop following, use {}",
                target_chat_id.0,
                CommandUnfollow.to_command_string(true)
            ))
            .await?;

        Ok(())
    }
}
