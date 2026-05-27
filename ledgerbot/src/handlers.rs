use std::sync::Arc;

use telluride::{
    command::CallbackKey,
    data_store::{InMemStore, UserProxy},
    markdown::MarkdownStringMessage,
    markdown_format,
};
use teloxide::{
    dispatching::DpHandlerDescription,
    prelude::*,
    types::{CallbackQuery, Chat, Me, MessageId, UserId},
    utils::command::BotCommands,
};

use crate::{
    batch::{add_to_batch, execute_batch},
    commands::{Command, execute_command},
    storages::Stores,
    utils::parse_expenses::parse_expenses,
};

pub fn filter_command_prefixed<C, Output>() -> dptree::Handler<'static, Output, DpHandlerDescription>
where
    C: BotCommands + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    dptree::filter_map(move |message: Message, me: Me| {
        let bot_name = me.user.username.expect("Bots must have a username");
        let text = message.text().or_else(|| message.caption())?;
        C::parse(text, &bot_name).ok().or_else(|| {
            let prefix = format!("@{} ", bot_name);
            text.strip_prefix(&prefix)
                .and_then(|stripped| C::parse(stripped, &bot_name).ok())
        })
    })
}

pub fn is_direct_command_message(msg: &Message) -> bool {
    if msg.text().is_none() || msg.forward_date().is_some() {
        return false;
    }

    msg.text()
        .map(|text| text.lines().filter(|line| !line.trim().is_empty()).count() == 1)
        .unwrap_or(false)
}

async fn execute_and_report_command(
    bot: Bot,
    chat: Chat,
    msg_id: Option<MessageId>,
    user_id: UserId,
    storage: Arc<Stores>,
    callback_storage: Arc<InMemStore<CallbackKey, Command>>,
    cmd: Command,
    batch: bool,
) -> ResponseResult<()> {
    if let Err(e) = execute_command(
        bot.clone(),
        chat.clone(),
        msg_id,
        user_id,
        storage,
        callback_storage,
        cmd.clone(),
        batch,
    )
    .await
    {
        log::error!("Failed to execute command `{}`: {}", cmd, e);
        bot.send_markdown_message(
            chat.id,
            markdown_format!(
                "❌ Error executing command `{}`: {}",
                cmd.to_string(),
                e.to_string()
            ),
        )
        .await?;
    }

    Ok(())
}

pub async fn handle_command_message(
    bot: Bot,
    msg: Message,
    cmd: Command,
    storage: Arc<Stores>,
    callback_storage: Arc<InMemStore<CallbackKey, Command>>,
) -> ResponseResult<()> {
    let user_id = msg
        .from
        .as_ref()
        .map(|user| user.id)
        .unwrap_or_else(|| UserId(msg.chat.id.0.unsigned_abs()));
    execute_and_report_command(
        bot,
        msg.chat.clone(),
        None,
        user_id,
        storage,
        callback_storage,
        cmd,
        false,
    )
    .await
}

/// Handle text messages containing potential expense data
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: Arc<Stores>,
    callback_storage: Arc<InMemStore<CallbackKey, Command>>,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        // Get bot username for filtering
        let bot_name = bot.get_me().await.ok().map(|me| me.username().to_string());

        // Get message timestamp (Unix timestamp in seconds)
        // Use forward_date if available (for forwarded messages), otherwise use msg.date
        let timestamp = msg.forward_date().unwrap_or(msg.date).timestamp();

        // Parse commands from the message, with bot name filtering and timestamp
        // Text expenses are now converted to Command::Expense variants
        let parsed_results = parse_expenses(text, bot_name.as_deref(), timestamp);

        log::info!(
            "Parsed {} results from chat {}",
            parsed_results.len(),
            msg.chat.id
        );

        // Check if we should process this message in batch mode
        let is_multiline = text.lines().filter(|line| !line.trim().is_empty()).count() > 1;
        let is_forwarded = msg.forward_date().is_some();

        // For multiline or forwarded messages, collect commands for batch execution.
        // For single-line, non-forwarded messages, execute immediately.
        if is_multiline || is_forwarded {
            let user_id = msg
                .from
                .as_ref()
                .map(|user| user.id)
                .unwrap_or_else(|| UserId(msg.chat.id.0.unsigned_abs()));
            // Add to batch storage for deferred execution
            let batch_storage = storage.storage(msg.chat.id).batch();
            let is_first_message = add_to_batch(batch_storage.clone(), parsed_results).await;

            // Start timeout task only for the first message in batch
            if is_first_message {
                let bot_clone = bot.clone();
                let storage_clone = storage.clone();
                let callback_storage_clone = callback_storage.clone();
                tokio::spawn(async move {
                    execute_batch(
                        bot_clone,
                        batch_storage,
                        msg.chat.clone(),
                        storage_clone,
                        callback_storage_clone,
                        user_id,
                    )
                    .await;
                });
            }
        } else {
            // Single-line message: execute immediately (existing behavior)
            for result in parsed_results {
                match result {
                    Ok(cmd) => {
                        execute_and_report_command(
                            bot.clone(),
                            msg.chat.clone(),
                            None,
                            msg.from
                                .as_ref()
                                .map(|user| user.id)
                                .unwrap_or_else(|| UserId(msg.chat.id.0.unsigned_abs())),
                            storage.clone(),
                            callback_storage.clone(),
                            cmd,
                            false,
                        )
                        .await?;
                    }
                    Err(err_msg) => {
                        // Send error message to user
                        log::warn!("Parse error in chat {}: {}", msg.chat.id, err_msg);
                        bot.send_markdown_message(msg.chat.id, markdown_format!("❌ {}", err_msg))
                            .await?;
                    }
                }
            }

            // For single-line messages with expenses, we don't batch - already executed above
        }
    }

    Ok(())
}

/// Handle callback queries from inline keyboard buttons
pub async fn handle_callback_query(
    bot: Bot,
    q: CallbackQuery,
    storage: Arc<Stores>,
    callback_storage: Arc<InMemStore<CallbackKey, Command>>,
) -> ResponseResult<()> {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(message) = q.message else {
        return Ok(());
    };

    let Some(msg) = message.regular_message() else {
        return Ok(());
    };

    let msg = msg.clone();
    let Some(data) = &q.data else {
        return Ok(());
    };

    let user_proxy = UserProxy::new(callback_storage.clone(), q.from.id);
    let Ok(cmd) = CallbackKey::unpack::<Command, _>(data, &user_proxy).await else {
        return Ok(());
    };

    let mut msg = msg;
    msg.from = Some(q.from.clone());

    execute_and_report_command(
        bot.clone(),
        msg.chat.clone(),
        Some(msg.id),
        q.from.id,
        storage.clone(),
        callback_storage,
        cmd,
        false,
    )
    .await?;

    Ok(())
}
