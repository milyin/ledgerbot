use std::sync::Arc;

use telluride::{
    command::{CallbackEncode, CallbackKey, InlineKeyboardButtonPackedExt},
    data_store::{InMemStore, UserProxy},
    markdown::{MarkdownString, MarkdownStringMessage},
};
use teloxide::utils::command::BotCommands;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::*,
    types::{Chat, InlineKeyboardButton, InlineKeyboardMarkup, MessageId, UserId},
};

use crate::commands::Command;

#[derive(Clone)]
pub enum ButtonData {
    Command(String, Command),
    RawCallback(String, String),
    SwitchInlineQuery(String, String),
}

pub struct CommandContext {
    pub bot: Bot,
    pub chat: Chat,
    pub message_id: Option<MessageId>,
    pub user_id: UserId,
    pub batch: bool,
    pub callback_storage: Arc<InMemStore<CallbackKey, Command>>,
}

impl CommandContext {
    pub async fn markdown_message(&self, text: MarkdownString) -> ResponseResult<Message> {
        if let Some(message_id) = self.message_id {
            self.bot
                .edit_markdown_message_text(self.chat.id, message_id, text)
                .await
        } else {
            self.bot.send_markdown_message(self.chat.id, text).await
        }
    }

    pub fn send_markdown_message(
        &self,
        text: MarkdownString,
    ) -> teloxide::requests::JsonRequest<teloxide::payloads::SendMessage> {
        self.bot.send_markdown_message(self.chat.id, text)
    }

    pub async fn update_markdown_message(
        &self,
        text: MarkdownString,
        keyboard: Option<InlineKeyboardMarkup>,
    ) -> ResponseResult<()> {
        match self.message_id {
            Some(id) => {
                let req = self.bot.edit_markdown_message_text(self.chat.id, id, text);
                let result = if let Some(kb) = keyboard {
                    req.reply_markup(kb).await.map(|_| ())
                } else {
                    req.await.map(|_| ())
                };
                match result {
                    Ok(()) => {}
                    Err(teloxide::RequestError::Api(teloxide::ApiError::MessageNotModified)) => {}
                    Err(e) => return Err(e),
                }
            }
            None => {
                let req = self.bot.send_markdown_message(self.chat.id, text);
                if let Some(kb) = keyboard {
                    req.reply_markup(kb).await?;
                } else {
                    req.await?;
                }
            }
        }
        Ok(())
    }

    pub async fn keyboard(&self, rows: Vec<Vec<ButtonData>>) -> InlineKeyboardMarkup {
        let user_proxy = UserProxy::new(self.callback_storage.clone(), self.user_id);
        let mut keyboard = Vec::new();

        for row in rows {
            let mut button_row = Vec::new();
            for button in row {
                let button = match button {
                    ButtonData::Command(label, command) => {
                        let key = CallbackKey::pack(command, &user_proxy).await;
                        InlineKeyboardButton::callback_key(label, &key)
                    }
                    ButtonData::RawCallback(label, data) => {
                        InlineKeyboardButton::callback(label, data)
                    }
                    ButtonData::SwitchInlineQuery(label, query) => {
                        InlineKeyboardButton::switch_inline_query_current_chat(label, query)
                    }
                };
                button_row.push(button);
            }
            keyboard.push(button_row);
        }

        InlineKeyboardMarkup::new(keyboard)
    }

    pub async fn markdown_message_with_menu(
        &self,
        text: MarkdownString,
        rows: Vec<Vec<ButtonData>>,
    ) -> ResponseResult<Message> {
        let message = self.markdown_message(text).await?;
        let keyboard = self.keyboard(rows).await;
        self.bot
            .edit_message_reply_markup(self.chat.id, message.id)
            .reply_markup(keyboard)
            .await?;
        Ok(message)
    }
}

impl CallbackEncode for Command {
    fn encode_callback(&self) -> Vec<u8> {
        self.to_command_string(false).into_bytes()
    }

    fn decode_callback(bytes: &[u8]) -> Result<Self, telluride::command::UnpackError> {
        let command = String::from_utf8(bytes.to_vec())
            .map_err(|e| telluride::command::UnpackError::DeserializeError(e.to_string()))?;
        Command::parse(&command, "")
            .map_err(|e| telluride::command::UnpackError::DeserializeError(e.to_string()))
    }
}
