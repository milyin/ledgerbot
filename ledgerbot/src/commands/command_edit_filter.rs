use core::task;
use std::sync::Arc;

use teloxide::{
    Bot,
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{Chat, InlineKeyboardButton, InlineKeyboardMarkup, MessageId},
};
use yoroolbot::{markdown::MarkdownStringMessage, markdown_format, markdown_string};

use crate::{
    commands::{
        command_add_category::CommandAddCategory,
        command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    },
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandEditFilter {
    pub category: Option<String>,
    pub position: Option<usize>,
    pub pattern: Option<String>,
}

impl CommandTrait for CommandEditFilter {
    type A = String;
    type B = usize;
    type C = String;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "edit_filter";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<position>", "<new_pattern>"];

    fn from_arguments(
        a: Option<Self::A>,
        b: Option<Self::B>,
        c: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandEditFilter {
            category: a,
            position: b,
            pattern: c,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }
    fn param2(&self) -> Option<&Self::B> {
        self.position.as_ref()
    }
    fn param3(&self) -> Option<&Self::C> {
        self.pattern.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        let categories = storage.get_chat_categories(target.chat.id).await;
        if categories.is_empty() {
            target
                .send_markdown_message(markdown_format!(
                    "📂 No categories defined yet\\. Use {} to create one\\.",
                    CommandAddCategory::default().to_command_string(true)
                ))
                .await?;
            return Ok(());
        }
        let msg = target
            .markdown_message(markdown_string!("✏️ Select Category for editing filter"))
            .await?;
        let menu = create_categories_menu(
            &categories.keys().cloned().collect::<Vec<_>>(),
            |name| {
                CommandEditFilter {
                    category: Some(name.to_string()),
                    position: None,
                    pattern: None,
                }
                .to_command_string(false)
            },
            None::<NoopCommand>,
            false,
        );
        target
            .bot
            .edit_message_reply_markup(target.chat.id, msg.id)
            .reply_markup(menu)
            .await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
    ) -> ResponseResult<()> {
        let Some(patterns) = read_category(
            target,
            &storage,
            name,
            Some(CommandEditFilter::default().to_command_string(false)),
        )
        .await?
        else {
            return Ok(());
        };
        let msg = target
            .markdown_message(markdown_format!("✏️ Edit filters in category `{}`", name))
            .await?;
        let menu = create_category_filters_menu(
            &patterns,
            |idx| {
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: Some(idx),
                    pattern: patterns.get(idx).cloned(),
                }
                .to_command_string(false)
            },
            Some(CommandEditFilter::default()),
            true,
        );
        target
            .bot
            .edit_message_reply_markup(target.chat.id, msg.id)
            .reply_markup(menu)
            .await?;
        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
        idx: &usize,
    ) -> ResponseResult<()> {
        let Some(pattern) = read_category_filter(
            &target,
            &storage,
            name,
            *idx,
            Some(
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: None,
                    pattern: None,
                }
                .to_command_string(false),
            ),
        )
        .await?
        else {
            return Ok(());
        };

        let msg = target
            .markdown_message(markdown_format!(
                "✏️ **Editing filter \\#{} in category `{}`:**\n\nCurrent pattern: `{}`",
                *idx,
                name,
                &pattern
            ))
            .await?;
        let menu = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::switch_inline_query_current_chat(
                "✏️ Edit pattern",
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: Some(*idx),
                    pattern: Some(pattern),
                }
                .to_command_string(false),
            )],
            vec![InlineKeyboardButton::callback(
                "↩️ Back",
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: None,
                    pattern: None,
                }
                .to_command_string(false),
            )],
        ]);
        target
            .bot
            .edit_message_reply_markup(target.chat.id, msg.id)
            .reply_markup(menu)
            .await?;

        Ok(())
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
        idx: &usize,
        pattern: &String,
    ) -> ResponseResult<()> {
        let Some(old_pattern) = read_category_filter(
            target,
            &storage,
            name,
            *idx,
            Some(
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: None,
                    pattern: None,
                }
                .to_command_string(false),
            ),
        )
        .await?
        else {
            return Ok(());
        };

        if let Err(e) = regex::Regex::new(&pattern) {
            let msg = target
                .markdown_message(markdown_format!(
                    "❌ Invalid regex pattern `{}`:\n{}",
                    &(*pattern),
                    &e.to_string()
                ))
                .await?;
            // append back button
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                CommandEditFilter {
                    category: Some(name.clone()),
                    position: Some(*idx),
                    pattern: None,
                }
                .to_command_string(false),
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
            return Ok(());
        }

        // Remove the old pattern and add the new one
        storage
            .remove_category_filter(target.chat.id, &name, &old_pattern)
            .await;

        storage
            .add_category_filter(target.chat.id, name.clone(), pattern.clone())
            .await;

        let msg = target
            .markdown_message(markdown_format!(
                "✅ Filter updated in category `{}`\\.\n*Old:* `{}`\n*New:* `{}`",
                name.clone(),
                old_pattern.clone(),
                pattern.clone()
            ))
            .await?;
        // append back button
        let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            "↩️ Back",
            CommandEditFilter {
                category: Some(name.clone()),
                position: None,
                pattern: None,
            }
            .to_command_string(false),
        )]]);
        target
            .bot
            .edit_message_reply_markup(target.chat.id, msg.id)
            .reply_markup(menu)
            .await?;

        Ok(())
    }
}

pub async fn read_category(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    back: Option<String>,
) -> ResponseResult<Option<Vec<String>>> {
    let categories = storage.get_chat_categories(target.chat.id).await;
    let Some(filters) = categories.get(name) else {
        let msg = target
            .markdown_message(markdown_format!("❌ Category `{}` does not exist", name))
            .await?;
        if let Some(back) = back {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back,
            )]]);
            target
                .bot
                .edit_message_reply_markup(target.chat.id, msg.id)
                .reply_markup(menu)
                .await?;
        }
        return Ok(None);
    };
    Ok(Some(filters.clone()))
}

pub async fn read_category_filter(
    target: &CommandReplyTarget,
    storage: &Arc<dyn CategoryStorageTrait>,
    name: &str,
    idx: usize,
    back: Option<String>,
) -> ResponseResult<Option<String>> {
    let Some(filters) = read_category(target, storage, name, back.clone()).await? else {
        return Ok(None);
    };
    if idx >= filters.len() {
        let msg = target
            .markdown_message(markdown_format!("❌ Invalid filter position `{}`", idx))
            .await?;
        if let Some(back) = back {
            let menu = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                "↩️ Back",
                back,
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

pub fn create_category_filters_menu(
    filters: &[String],
    operation: impl Fn(usize) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = filters
        .iter()
        .enumerate()
        .map(|(idx, pattern)| format!("{}. {}", idx, pattern))
        .collect::<Vec<_>>();
    let values = filters
        .iter()
        .enumerate()
        .map(|(idx, _)| operation(idx))
        .collect::<Vec<_>>();
    // use create_menu
    create_buttons_menu(&texts, &values, back_command, inline)
}

pub fn create_categories_menu(
    categories: &[String],
    operation: impl Fn(&str) -> String,
    back_command: Option<impl CommandTrait>,
    inline: bool,
) -> InlineKeyboardMarkup {
    let texts = categories
        .iter()
        .map(|name| format!("📁 {}", name))
        .collect::<Vec<_>>();
    let values = categories
        .iter()
        .map(|name| operation(name))
        .collect::<Vec<_>>();
    create_buttons_menu(&texts, &values, back_command, inline)
}
