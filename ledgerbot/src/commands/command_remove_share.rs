use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{
    menus::{select_share::select_share, update_share::update_share},
    storages::{ShareStorageTrait, ShareUsername},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandRemoveShare {
    pub username: Option<ShareUsername>,
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandRemoveShare {
    type A = ShareUsername;
    type B = bool;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn ShareStorageTrait>;

    const NAME: &'static str = "remove_share";
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
        CommandRemoveShare { username, confirm }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_share(
            target,
            &storage,
            markdown_string!("✏️ Select username to remove"),
            |username| CommandRemoveShare {
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
        username: &ShareUsername,
    ) -> ResponseResult<()> {
        update_share(
            target,
            &storage,
            username,
            markdown_format!("🗑️ Confirm Username `{}` Removal", username.as_str()),
            "🗑️ Remove",
            CommandRemoveShare {
                username: Some(username.clone()),
                confirm: Some(true),
            },
            Some(CommandRemoveShare {
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
        username: &ShareUsername,
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

        if let Err(e) = storage.remove_share(username).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!(
                "✅ Username `{}` removed from share list\\.",
                username.as_str()
            ))
            .await?;
        Ok(())
    }
}
