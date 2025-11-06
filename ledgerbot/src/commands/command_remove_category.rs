use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{
    menus::{select_category::select_category, update_category::update_category},
    storages::{Category, CategoryStorageTrait},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandRemoveCategory {
    pub category: Option<Category>,
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandRemoveCategory {
    type A = Category;
    type B = bool;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "remove_category";
    const PLACEHOLDERS: &[&'static str] = &["<name>", "<confirm>"];

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.confirm.as_ref()
    }

    fn from_arguments(
        category: Option<Self::A>,
        confirm: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandRemoveCategory { category, confirm }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("✏️ Select Category to remove"),
            |category| CommandRemoveCategory {
                category: Some(category.clone()),
                confirm: None,
            },
            None::<NoopCommand>,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &Category,
    ) -> ResponseResult<()> {
        update_category(
            target,
            &storage,
            category,
            markdown_format!("🗑️ Confirm Category `{}` Removal", category.as_str()),
            "🗑️ Remove",
            CommandRemoveCategory {
                category: Some(category.clone()),
                confirm: Some(true),
            },
            Some(CommandRemoveCategory {
                category: None,
                confirm: None,
            }),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &Category,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Category `{}` removal cancelled\\.",
                    category.as_str()
                ))
                .await?;
            return Ok(());
        }

        if let Err(e) = storage.remove_category(category).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!(
                "✅ Category `{}` removed\\.",
                category.as_str()
            ))
            .await?;
        Ok(())
    }
}
