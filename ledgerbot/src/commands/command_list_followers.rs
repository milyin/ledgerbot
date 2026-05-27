use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::markdown_format;
use teloxide::prelude::{Requester, ResponseResult};

use crate::{
    commands::{command_add_follower::CommandAddFollower, command_follow::CommandFollow},
    impl_command_execute_0, impl_command_io,
    storages::FollowersStorageTrait,
    ui::CommandContext,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandListFollowers;

impl_command_io!(CommandListFollowers, "list_followers", []);
impl_command_execute_0!(CommandListFollowers, Arc<dyn FollowersStorageTrait>);

impl CommandListFollowers {
    async fn run0(
        &self,
        target: &CommandContext,
        storage: Arc<dyn FollowersStorageTrait>,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let followers = storage.get_followers().await.unwrap_or_default();

        if followers.is_empty() {
            target
                .send_markdown_message(markdown_format!(
                    "👥 No usernames in followers list yet\\. Use {} to add one\\.",
                    CommandAddFollower::default().to_command_string(true)
                ))
                .await?;
        } else {
            target
                .send_markdown_message(markdown_format!(
                    "👥 The users below can use command `{}` to follow expenses from this chat",
                    CommandFollow::new(chat_id).to_command_string(true),
                ))
                .await?;

            let mut result = String::new();
            let mut sorted_followers = followers;
            sorted_followers.sort();

            for username in sorted_followers {
                result
                    .push_str(&CommandAddFollower::new(username.as_str()).to_command_string(true));
                result.push('\n');
            }

            target.bot.send_message(chat_id, result).await?;
        }

        Ok(())
    }
}

impl From<CommandListFollowers> for crate::commands::Command {
    fn from(cmd: CommandListFollowers) -> Self {
        crate::commands::Command::ListFollowers(cmd)
    }
}
