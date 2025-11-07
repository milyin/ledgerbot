use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::prelude::{Requester, ResponseResult};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{
    commands::{command_add_follower::CommandAddFollower, command_follow::CommandFollow},
    storages::FollowersStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandListFollowers;

impl CommandTrait for CommandListFollowers {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn FollowersStorageTrait>;

    const NAME: &'static str = "list_followers";
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
        CommandListFollowers
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
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

            let mut result = "".to_string();
            // Sort usernames for consistent output
            let mut sorted_followers = followers;
            sorted_followers.sort();

            for username in sorted_followers {
                result.push_str(&CommandAddFollower::new(username.as_str()).to_command_string(true));
                result.push('\n');
            }

            target.bot.send_message(chat_id, result).await?;
        }

        Ok(())
    }
}
