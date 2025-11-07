use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::{prelude::ResponseResult, types::ChatId};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::storages::Stores;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandUnfollow;

impl CommandTrait for CommandUnfollow {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<Stores>;

    const NAME: &'static str = "unfollow";
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
        CommandUnfollow
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
            let storage_ = storage.storage(chat_id);
        let variable_storage = storage_.variables();

        // Check if currently following any chat
        let followed_chat: Option<ChatId> = variable_storage.get().await;

        match followed_chat {
            Some(chat_id) => {
                // Remove the follow relationship
                variable_storage
                    .remove::<Option<ChatId>>()
                    .await;

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
