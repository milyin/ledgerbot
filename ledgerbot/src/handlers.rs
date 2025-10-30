use std::sync::Arc;

use teloxide::{prelude::*, types::CallbackQuery, utils::command::BotCommands};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format, storage::unpack_callback_data};

use crate::{
    batch::{add_to_batch, execute_batch},
    commands::{Command, execute_command},
    storages::StorageTrait,
    utils::parse_expenses::parse_expenses,
};

/// Handle text messages containing potential expense data
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: Arc<dyn StorageTrait>,
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
            // Add to batch storage for deferred execution
            let batch_storage = storage.clone().as_batch_storage();
            let is_first_message =
                add_to_batch(batch_storage.clone(), msg.chat.clone(), parsed_results).await;

            // Start timeout task only for the first message in batch
            if is_first_message {
                let bot_clone = bot.clone();
                let storage_clone = storage.clone();
                tokio::spawn(async move {
                    execute_batch(bot_clone, batch_storage, msg.chat.clone(), storage_clone).await;
                });
            }
        } else {
            // Single-line message: execute immediately (existing behavior)
            for result in parsed_results {
                match result {
                    Ok(cmd) => {
                        // Execute the command using the shared execute_command function
                        let exec_result = execute_command(
                            bot.clone(),
                            msg.chat.clone(),
                            None,
                            storage.clone(),
                            cmd,
                            false,
                        )
                        .await;
                        if let Err(e) = exec_result {
                            log::error!("Failed to execute command: {}", e);
                            bot.markdown_message(
                                msg.chat.id,
                                None,
                                markdown_format!("❌ Error: {}", e.to_string()),
                            )
                            .await?;
                        }
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
    storage: Arc<dyn StorageTrait>,
) -> ResponseResult<()> {
    let bot_username = bot.get_me().await?.username().to_string();
    // Answer the callback query to remove the loading state
    bot.answer_callback_query(q.id.clone()).await?;

    // Get the message that contained the button
    let Some(message) = q.message else {
        return Ok(());
    };

    let Some(msg) = message.regular_message() else {
        return Ok(());
    };

    let msg = msg.clone();
    let chat_id = msg.chat.id;

    // Parse callback data string into enum
    let Some(data_str) = &q.data else {
        return Ok(());
    };

    log::info!("Received callback data: {}", data_str);

    // Unpack callback data from storage if needed
    let callback_storage = storage.clone().as_callback_data_storage();
    let unpacked_data = unpack_callback_data(&callback_storage, data_str).await;

    log::info!("Unpacked callback data: {}", unpacked_data);

    // Try to parse the callback data as command
    if let Ok(cmd) = Command::parse(&unpacked_data, &bot_username) {
        log::info!("Parsed command from callback: {:?}", cmd);
        // Execute the command using the shared execute_command function
        if let Err(e) = execute_command(
            bot.clone(),
            msg.chat.clone(),
            Some(msg.id),
            storage.clone(),
            cmd.clone(),
            false,
        )
        .await
        {
            log::error!("Failed to execute command from callback: {}", e);
            bot.send_markdown_message(
                chat_id,
                markdown_format!(
                    "❌ Error executing command `{}`: {}",
                    cmd.to_string(),
                    e.to_string()
                ),
            )
            .await?;
        }
        return Ok(());
    }

    Ok(())
}
