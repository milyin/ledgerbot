use teloxide::prelude::*;

use crate::batch::{add_to_batch, send_batch_report, BatchStorage};
use crate::parser::parse_expenses;
use crate::storage::{add_expenses, ExpenseStorage};

/// Handle text messages containing potential expense data
pub async fn handle_text_message(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
    batch_storage: BatchStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    if let Some(text) = msg.text() {
        // Parse expenses from the message
        let parsed_expenses = parse_expenses(text);

        if !parsed_expenses.is_empty() {
            // Store the expenses in chat-specific storage
            add_expenses(&storage, chat_id, parsed_expenses.clone()).await;

            // Update batch state for this chat
            let total_parsed: f64 = parsed_expenses.iter().map(|(_, amount)| amount).sum();
            let is_first_message =
                add_to_batch(&batch_storage, chat_id, parsed_expenses.len(), total_parsed).await;

            // Start timeout task only for the first message in batch
            if is_first_message {
                let batch_clone = batch_storage.clone();
                let bot_clone = bot.clone();
                tokio::spawn(async move {
                    send_batch_report(bot_clone, batch_clone, chat_id).await;
                });
            }
        }
    }

    Ok(())
}
