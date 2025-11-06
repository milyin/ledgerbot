use std::sync::Arc;

use teloxide::{prelude::ResponseResult, types::ChatId};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{
    commands::{command_unfollow::CommandUnfollow, follow_helper},
    storages::StorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandFollow {
    pub chat_id: Option<i64>,
}

impl CommandFollow {
    pub fn new(chat_id: ChatId) -> Self {
        CommandFollow {
            chat_id: Some(chat_id.0),
        }
    }
}

impl CommandTrait for CommandFollow {
    type A = i64;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "follow";
    const PLACEHOLDERS: &[&'static str] = &["<chat_id>"];

    fn param1(&self) -> Option<&Self::A> {
        self.chat_id.as_ref()
    }

    fn from_arguments(
        chat_id: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandFollow { chat_id }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let variable_storage = storage.clone().as_variable_storage();

        // Check if currently following any chat
        let followed_chat: Option<ChatId> = variable_storage.get(target.chat.id).await;

        let status_message = match followed_chat {
            Some(chat_id) => {
                // Validate current follow access
                match follow_helper::validate_and_get_follow_access(target, &storage).await {
                    Ok(_) => {
                        markdown_format!(
                            "👁️ **Currently following:** chat `{}`\n\n\
                             To change, use `{}`\n\
                             To stop following, use `{}`",
                            chat_id.0,
                            CommandFollow::default().to_command_string(true),
                            CommandUnfollow.to_command_string(true)
                        )
                    }
                    Err(warning_msg) => {
                        // Access lost, show warning and usage
                        warning_msg
                            + markdown_format!(
                                "\n\nℹ️ Usage: `{}`\n\n\
                                 Follow expenses from another chat that has shared access to you\\.",
                                CommandFollow::default().to_command_string(true)
                            )
                    }
                }
            }
            None => {
                markdown_format!(
                    "ℹ️ Not currently following any chat\\.\n\n\
                     Usage: `{}`\n\n\
                     Follow expenses from another chat that has shared access to you\\.",
                    CommandFollow::default().to_command_string(true)
                )
            }
        };

        target.send_markdown_message(status_message).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        target_chat_id: &i64,
    ) -> ResponseResult<()> {
        let target_chat_id = ChatId(*target_chat_id);

        // Validate access using the shared helper
        if let Err(error_msg) =
            follow_helper::validate_follow_access(target, &storage, target_chat_id).await
        {
            target.send_markdown_message(error_msg).await?;
            return Ok(());
        }

        // User is authorized - store the follow relationship
        let variable_storage = storage.clone().as_variable_storage();
        variable_storage.set(target.chat.id, target_chat_id).await;

        let q = variable_storage.get::<ChatId>(target.chat.id).await;
        log::info!("Set follow chat for {}: {:?}", target.chat.id, q);

        target
            .send_markdown_message(markdown_format!(
                "✅ Now following expenses from chat `{}`\\.\n\nTo stop following, use {}",
                target_chat_id.0,
                CommandUnfollow.to_command_string(true)
            ))
            .await?;

        Ok(())
    }
}
