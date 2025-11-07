use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format, markdown_string,
};

use crate::{
    commands::command_follow::CommandFollow,
    storages::{FollowersStorageTrait, TelegramUsername},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl CommandTrait for CommandAddFollower {
    type A = TelegramUsername;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn FollowersStorageTrait>;

    const NAME: &'static str = "add_follower";
    const PLACEHOLDERS: &[&'static str] = &["<username>"];

    fn from_arguments(
        a: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddFollower { username: a }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.username.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _storage: Self::Context,
    ) -> teloxide::prelude::ResponseResult<()> {
        target
            .send_markdown_message(markdown_string!("➕ Add Follower"))
            .await?;
        add_follower_menu(target).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        username: &TelegramUsername,
    ) -> teloxide::prelude::ResponseResult<()> {
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

/// Show add share menu
pub async fn add_follower_menu(target: &CommandReplyTarget) -> ResponseResult<()> {
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
