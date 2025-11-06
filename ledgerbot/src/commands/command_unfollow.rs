use std::sync::Arc;

use teloxide::{prelude::ResponseResult, types::ChatId};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::storages::StorageTrait;

#[derive(Default, Debug, Clone, PartialEq)]
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

    type Context = Arc<dyn StorageTrait>;

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
        let variable_storage = storage.clone().as_variable_storage();

        // Check if currently following any chat
        let followed_chat: Option<ChatId> = variable_storage.get(target.chat.id).await;

        match followed_chat {
            Some(chat_id) => {
                // Remove the follow relationship
                variable_storage
                    .remove::<Option<ChatId>>(target.chat.id)
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
                    .send_markdown_message(markdown_format!(
                        "ℹ️ You are not following any chat\\."
                    ))
                    .await?;
            }
        }

        Ok(())
    }
}
