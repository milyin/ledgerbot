use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format, markdown_string,
};

use crate::storages::{ShareStorageTrait, ShareUsername};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddShare {
    pub username: Option<ShareUsername>,
}

impl CommandAddShare {
    pub fn new(username: impl Into<String>) -> Self {
        CommandAddShare {
            username: ShareUsername::from_string(&username.into()).ok(),
        }
    }
}

impl CommandTrait for CommandAddShare {
    type A = ShareUsername;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn ShareStorageTrait>;

    const NAME: &'static str = "add_share";
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
        CommandAddShare { username: a }
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
            .send_markdown_message(markdown_string!("➕ Add Share"))
            .await?;
        add_share_menu(target).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        username: &ShareUsername,
    ) -> teloxide::prelude::ResponseResult<()> {
        match storage.add_share(target.chat.id, username).await {
            Ok(()) => {
                target
                    .send_markdown_message(markdown_format!(
                        "✅ Username `{}` added to share list\\.",
                        username.as_str()
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
pub async fn add_share_menu(target: &CommandReplyTarget) -> ResponseResult<()> {
    let text = markdown_string!(
        "➕ **Add a username to share list:**\n\nClick the button below and type the username \\(e\\.g\\., @username\\)\\."
    );
    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::switch_inline_query_current_chat(
            "➕ Add Share",
            CommandAddShare::default().to_command_string(false),
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
