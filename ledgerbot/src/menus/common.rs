use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait},
    markdown_format,
};

use crate::storages::storage_traits::CategoryStorageTrait;

pub fn create_buttons_menu(
    titles: &[String],
    values: &[String],
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = titles
        .iter()
        .zip(values.iter())
        .map(|(text, value)| {
            if inline {
                vec![InlineKeyboardButton::switch_inline_query_current_chat(
                    text,
                    value.clone(),
                )]
            } else {
                vec![InlineKeyboardButton::callback(text, value.clone())]
            }
        })
        .collect();
    if let Some(back) = back_command {
        buttons.push(vec![InlineKeyboardButton::callback(
            "↩️ Back",
            back.to_command_string(false),
        )]);
    }
    InlineKeyboardMarkup::new(buttons)
}

pub async fn read_category_filters_list(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    back_command: Option<impl CommandTrait>,
) -> ResponseResult<Vec<String>> {
    let categories = storage
        .get_chat_categories(target.chat.id)
        .await
        .unwrap_or_default();
    let Some(filters) = categories.get(name) else {
        let msg = target
            .markdown_message(markdown_format!("❌ Category `{}` does not exist", name))
            .await?;
        if let Some(back) = back_command {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(Vec::new());
    };
    if filters.is_empty() {
        let msg = target
            .markdown_message(markdown_format!(
                "📂 Category `{}` has no filters defined yet\\.",
                name
            ))
            .await?;
        if let Some(back) = back_command {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(Vec::new());
    }
    Ok(filters.clone())
}

pub async fn read_category_filter_by_index(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    idx: usize,
    back_command: Option<impl CommandTrait>,
) -> ResponseResult<Option<String>> {
    let filters = read_category_filters_list(target, storage, name, back_command.clone()).await?;
    if filters.is_empty() {
        return Ok(None);
    };
    if idx >= filters.len() {
        let msg = target
            .markdown_message(markdown_format!("❌ Invalid filter position `{}`", idx))
            .await?;
        if let Some(back) = back_command {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back.to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(None);
    }
    Ok(Some(filters[idx].clone()))
}

#[cfg(test)]
mod tests {
    use teloxide::types::ChatId;
    use yoroolbot::storage::{
        CallbackDataStorage, CallbackDataStorageTrait, pack_callback_data, unpack_callback_data,
    };

    use super::*;

    #[tokio::test]
    async fn test_pack_unpack_callback_data() {
        let storage: Arc<dyn CallbackDataStorageTrait> = Arc::new(CallbackDataStorage::new());
        let chat_id = ChatId(12345);
        let message_id = 67890;

        // Create button data with short and long callback data
        let button_rows = vec![
            vec![
                ("Button 1".to_string(), "short".to_string()),
                (
                    "Button 2".to_string(),
                    "toggle_word:long_category_name:long_word_that_exceeds_64_bytes_limit_for_telegram_callback_data".to_string(),
                ),
            ],
            vec![
                ("Button 3".to_string(), "another_short".to_string()),
                ("Button 4".to_string(), "Кнопка с кириллицей".to_string()),
            ],
        ];

        // Pack the callback data
        let keyboard = pack_callback_data(&storage, chat_id, message_id, button_rows.clone()).await;

        // Verify keyboard structure
        assert_eq!(keyboard.inline_keyboard.len(), 2);
        assert_eq!(keyboard.inline_keyboard[0].len(), 2);
        assert_eq!(keyboard.inline_keyboard[1].len(), 2);

        // Get callback data strings from buttons
        let cb1 = match &keyboard.inline_keyboard[0][0].kind {
            teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => data.clone(),
            _ => panic!("Expected callback button"),
        };
        let cb2 = match &keyboard.inline_keyboard[0][1].kind {
            teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => data.clone(),
            _ => panic!("Expected callback button"),
        };
        let cb3 = match &keyboard.inline_keyboard[1][0].kind {
            teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => data.clone(),
            _ => panic!("Expected callback button"),
        };
        let cb4 = match &keyboard.inline_keyboard[1][1].kind {
            teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => data.clone(),
            _ => panic!("Expected callback button"),
        };

        // Verify short data is not stored (kept as-is)
        assert_eq!(cb1, "short");
        assert_eq!(cb3, "another_short");

        // Verify long data is stored (replaced with reference)
        assert!(cb2.starts_with("cb:"));
        assert!(cb4.starts_with("cb:"));

        // Unpack and verify
        let unpacked1 = unpack_callback_data(&storage, &cb1).await;
        let unpacked2 = unpack_callback_data(&storage, &cb2).await;
        let unpacked3 = unpack_callback_data(&storage, &cb3).await;
        let unpacked4 = unpack_callback_data(&storage, &cb4).await;

        assert_eq!(unpacked1, "short");
        assert_eq!(
            unpacked2,
            "toggle_word:long_category_name:long_word_that_exceeds_64_bytes_limit_for_telegram_callback_data"
        );
        assert_eq!(unpacked3, "another_short");
        assert_eq!(unpacked4, "Кнопка с кириллицей");
    }

    #[tokio::test]
    async fn test_pack_callback_data_clears_old_data() {
        let storage: Arc<dyn CallbackDataStorageTrait> = Arc::new(CallbackDataStorage::new());
        let chat_id = ChatId(12345);
        let message_id = 67890;

        // Create initial buttons with long callback data
        let initial_buttons = vec![vec![(
            "Button 1".to_string(),
            "toggle_word:category_name:very_long_word_that_exceeds_telegram_limit".to_string(),
        )]];

        // Pack initial buttons
        let initial_keyboard =
            pack_callback_data(&storage, chat_id, message_id, initial_buttons).await;

        let initial_cb = match &initial_keyboard.inline_keyboard[0][0].kind {
            teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => data.clone(),
            _ => panic!("Expected callback button"),
        };

        // Verify initial data is stored
        assert!(initial_cb.starts_with("cb:"));
        let initial_unpacked = unpack_callback_data(&storage, &initial_cb).await;
        assert_eq!(
            initial_unpacked,
            "toggle_word:category_name:very_long_word_that_exceeds_telegram_limit"
        );

        // Now pack new buttons for the same message (should clear old data)
        let new_buttons = vec![vec![(
            "Button 2".to_string(),
            "toggle_word:new_category:another_very_long_word_that_also_exceeds_limit".to_string(),
        )]];

        let new_keyboard = pack_callback_data(&storage, chat_id, message_id, new_buttons).await;

        let new_cb = match &new_keyboard.inline_keyboard[0][0].kind {
            teloxide::types::InlineKeyboardButtonKind::CallbackData(data) => data.clone(),
            _ => panic!("Expected callback button"),
        };

        // Verify new data is stored
        assert!(new_cb.starts_with("cb:"));
        let new_unpacked = unpack_callback_data(&storage, &new_cb).await;
        assert_eq!(
            new_unpacked,
            "toggle_word:new_category:another_very_long_word_that_also_exceeds_limit"
        );

        // Verify old reference now points to new data (since it uses same position)
        // This is correct behavior: when buttons are updated, old references are reused
        let old_ref_unpacked = unpack_callback_data(&storage, &initial_cb).await;
        assert_eq!(
            old_ref_unpacked,
            "toggle_word:new_category:another_very_long_word_that_also_exceeds_limit"
        );
    }
}
