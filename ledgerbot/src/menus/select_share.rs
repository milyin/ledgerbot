use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::InlineKeyboardMarkup,
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown::MarkdownString,
    markdown_format,
};

use crate::{
    commands::command_add_share::CommandAddShare,
    menus::common::create_buttons_menu,
    storages::{ShareStorageTrait, ShareUsername},
};

pub async fn select_share<NEXT: CommandTrait, BACK: CommandTrait>(
    target: &CommandReplyTarget,
    storage: &Arc<dyn ShareStorageTrait>,
    prompt: MarkdownString,
    next_command: impl Fn(&ShareUsername) -> NEXT,
    back_command: Option<BACK>,
) -> ResponseResult<()> {
    let shares = storage
        .get_shares()
        .await
        .unwrap_or_default();
    if shares.is_empty() {
        target
            .send_markdown_message(markdown_format!(
                "👥 No usernames in share list yet\\. Use {} to add one\\.",
                CommandAddShare::default().to_command_string(true)
            ))
            .await?;
        return Ok(());
    }

    let msg = target.markdown_message(prompt).await?;
    let menu = create_shares_menu(
        &shares,
        |username| next_command(username).to_command_string(false),
        back_command,
        false,
    );
    target
        .bot
        .edit_message_reply_markup(target.chat.id, msg.id)
        .reply_markup(menu)
        .await?;
    Ok(())
}

pub fn create_shares_menu(
    shares: &[ShareUsername],
    operation: impl Fn(&ShareUsername) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = shares
        .iter()
        .map(|username| format!("👤 {}", username.as_str()))
        .collect::<Vec<_>>();
    let values = shares.iter().map(operation).collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
