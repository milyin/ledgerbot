use std::sync::Arc;

use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    prelude::{Requester, ResponseResult},
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use yoroolbot::{markdown_format, markdown_string};

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    menus::{
        common::read_category_filter_by_index, select_category::select_category,
        select_category_filter::select_category_filter,
        update_category_filter::update_category_filter,
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
        select_category(
            target,
            &storage,
            markdown_string!("✏️ Select Category for editing filter"),
            |name| CommandEditFilter {
                category: Some(name.to_string()),
                position: None,
                pattern: None,
            },
            None::<NoopCommand>,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
    ) -> ResponseResult<()> {
        select_category_filter(
            target,
            &storage,
            name,
            markdown_format!("✏️ Select Filter to edit in category `{}`", name),
            |idx| CommandEditFilter {
                category: Some(name.clone()),
                position: Some(idx),
                pattern: None,
            },
            Some(CommandEditFilter {
                category: Some(name.clone()),
                position: None,
                pattern: None,
            }),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
        idx: &usize,
    ) -> ResponseResult<()> {
        update_category_filter(
            target,
            &storage,
            name,
            *idx,
            |pattern| {
                markdown_format!(
                    "✏️ **Editing filter \\#{} in category `{}`:**\n\nCurrent pattern: `{}`",
                    *idx,
                    name,
                    pattern
                )
            },
            "✏️ Edit pattern",
            |pattern| CommandEditFilter {
                category: Some(name.clone()),
                position: Some(*idx),
                pattern: Some(pattern.to_string()),
            },
            Some(CommandEditFilter {
                category: Some(name.clone()),
                position: None,
                pattern: None,
            }),
        )
        .await
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
        idx: &usize,
        pattern: &String,
    ) -> ResponseResult<()> {
        let Some(old_pattern) = read_category_filter_by_index(
            target,
            &storage,
            name,
            *idx,
            Some(CommandEditFilter {
                category: Some(name.clone()),
                position: None,
                pattern: None,
            }),
        )
        .await?
        else {
            return Ok(());
        };

        if let Err(e) = regex::Regex::new(pattern) {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Invalid regex pattern `{}`:\n{}",
                    pattern,
                    &e.to_string()
                ))
                .await?;
            return Ok(());
        }

        // Remove the old pattern and add the new one
        storage
            .remove_category_filter(target.chat.id, name, &old_pattern)
            .await;

        storage
            .add_category_filter(target.chat.id, name.clone(), pattern.clone())
            .await;

        target
            .send_markdown_message(markdown_format!(
                "✅ Filter updated in category `{}`\\.\n*Old:* `{}`\n*New:* `{}`",
                name.clone(),
                old_pattern.clone(),
                pattern.clone()
            ))
            .await?;

        Ok(())
    }
}
