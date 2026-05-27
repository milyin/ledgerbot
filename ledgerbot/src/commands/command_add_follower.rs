use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    commands::command_follow::CommandFollow,
    impl_command_execute_1, impl_command_io,
    storages::{FollowersStorageTrait, TelegramUsername},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandAddFollower {
    pub username: Option<TelegramUsername>,
}

impl CommandAddFollower {
    pub fn new(username: impl Into<String>) -> Self {
        CommandAddFollower {
            username: TelegramUsername::from_string(&username.into()).ok(),
        }
    }
}

impl_command_io!(CommandAddFollower, "add_follower", ["<username>"], username: TelegramUsername);
impl_command_execute_1!(CommandAddFollower, Arc<dyn FollowersStorageTrait>, username);

impl CommandAddFollower {
    async fn run0(
        &self,
        target: &CommandContext,
        _storage: Arc<dyn FollowersStorageTrait>,
    ) -> ResponseResult<()> {
        target
            .send_markdown_message(markdown_string!("➕ Add Follower"))
            .await?;
        add_follower_menu(target).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<dyn FollowersStorageTrait>,
        username: &TelegramUsername,
    ) -> ResponseResult<()> {
        match storage.add_follower(username).await {
            Ok(()) => {
                target
                    .send_markdown_message(markdown_format!(
                        "✅ Username `{}` added to followers list\\.\n\nUser `{}` can now follow this chat using `{}`",
                        username.as_str(),
                        username.as_str(),
                        CommandFollow {
                            chat_id: Some(target.chat.id.0),
                        }
                        .to_command_string(true)
                    ))
                    .await?;
            }
            Err(err_msg) => {
                target.send_markdown_message(err_msg).await?;
            }
        }
        Ok(())
    }
}

pub async fn add_follower_menu(target: &CommandContext) -> ResponseResult<()> {
    let text = markdown_string!(
        "➕ **Add a username to followers list:**\n\nClick the button below and type the username \\(e\\.g\\., @username\\)\\."
    );
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            "➕ Add Follower",
            CommandAddFollower::default().to_command_string(false),
        ),
    ]]);

    let message = target.markdown_message(text).await?;
    target
        .bot
        .edit_message_reply_markup(target.chat.id, message.id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

impl From<CommandAddFollower> for crate::commands::Command {
    fn from(cmd: CommandAddFollower) -> Self {
        crate::commands::Command::AddFollower(cmd)
    }
}
