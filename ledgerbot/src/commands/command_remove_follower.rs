use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::{markdown_format, markdown_string};
use teloxide::prelude::ResponseResult;

use crate::{
    commands::Command,
    impl_command_execute_2, impl_command_io,
    menus::{select_follower::select_follower, update_follower::update_follower},
    storages::{FollowersStorageTrait, TelegramUsername},
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandRemoveFollower {
    pub username: Option<TelegramUsername>,
    pub confirm: Option<bool>,
}

impl_command_io!(
    CommandRemoveFollower,
    "remove_follower",
    ["<username>", "<confirm>"],
    username: TelegramUsername,
    confirm: bool
);
impl_command_execute_2!(
    CommandRemoveFollower,
    Arc<dyn FollowersStorageTrait>,
    username,
    confirm
);

impl CommandRemoveFollower {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<dyn FollowersStorageTrait>,
    ) -> ResponseResult<()> {
        select_follower(
            target,
            &storage,
            markdown_string!("✏️ Select username to remove"),
            |username| {
                CommandRemoveFollower {
                    username: Some(username.clone()),
                    confirm: None,
                }
                .into()
            },
            None,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<dyn FollowersStorageTrait>,
        username: &TelegramUsername,
    ) -> ResponseResult<()> {
        update_follower(
            target,
            &storage,
            username,
            markdown_format!("🗑️ Confirm Username `{}` Removal", username.as_str()),
            "🗑️ Remove",
            CommandRemoveFollower {
                username: Some(username.clone()),
                confirm: Some(true),
            }
            .into(),
            Some(Command::RemoveFollower(CommandRemoveFollower {
                username: None,
                confirm: None,
            })),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandContext,
        storage: Arc<dyn FollowersStorageTrait>,
        username: &TelegramUsername,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Username `{}` removal cancelled\\.",
                    username.as_str()
                ))
                .await?;
            return Ok(());
        }

        if let Err(e) = storage.remove_follower(username).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!(
                "✅ Username `{}` removed from followers list\\.",
                username.as_str()
            ))
            .await?;
        Ok(())
    }
}

impl From<CommandRemoveFollower> for crate::commands::Command {
    fn from(cmd: CommandRemoveFollower) -> Self {
        crate::commands::Command::RemoveFollower(cmd)
    }
}
