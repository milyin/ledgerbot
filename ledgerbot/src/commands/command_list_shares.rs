use std::sync::Arc;

use teloxide::prelude::{Requester, ResponseResult};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{commands::command_add_share::CommandAddShare, storages::ShareStorageTrait};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandListShares;

impl CommandTrait for CommandListShares {
    type A = EmptyArg;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn ShareStorageTrait>;

    const NAME: &'static str = "list_shares";
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
        CommandListShares
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let chat_id = target.chat.id;
        let shares = storage.get_chat_shares(chat_id).await.unwrap_or_default();

        if shares.is_empty() {
            target
                .send_markdown_message(markdown_format!(
                    "👥 No usernames in share list yet\\. Use {} to add one\\.",
                    CommandAddShare::default().to_command_string(true)
                ))
                .await?;
        } else {
            let mut result = "".to_string();
            // Sort usernames for consistent output
            let mut sorted_shares = shares;
            sorted_shares.sort();

            for username in sorted_shares {
                result.push_str(&CommandAddShare::new(username.as_str()).to_command_string(true));
                result.push('\n');
            }

            target.bot.send_message(chat_id, result).await?;
        }

        Ok(())
    }
}
