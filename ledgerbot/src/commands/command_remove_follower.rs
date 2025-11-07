use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{
    menus::{select_follower::select_follower, update_follower::update_follower},
    storages::{FollowersStorageTrait, TelegramUsername},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandRemoveFollower {
    pub username: Option<TelegramUsername>,
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandRemoveFollower {
    type A = TelegramUsername;
    type B = bool;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn FollowersStorageTrait>;

    const NAME: &'static str = "remove_follower";
    const PLACEHOLDERS: &[&'static str] = &["<username>", "<confirm>"];

    fn param1(&self) -> Option<&Self::A> {
        self.username.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.confirm.as_ref()
    }

    fn from_arguments(
        username: Option<Self::A>,
        confirm: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandRemoveFollower { username, confirm }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_follower(
            target,
            &storage,
            markdown_string!("✏️ Select username to remove"),
            |username| CommandRemoveFollower {
                username: Some(username.clone()),
                confirm: None,
            },
            None::<NoopCommand>,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
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
            },
            Some(CommandRemoveFollower {
                username: None,
                confirm: None,
            }),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
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
