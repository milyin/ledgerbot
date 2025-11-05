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
    pub name: Option<String>,
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandRemoveCategory {
    type A = String;
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
        self.name.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.confirm.as_ref()
    }

    fn from_arguments(
        name: Option<Self::A>,
        confirm: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandRemoveCategory { name, confirm }
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
                name: Some(category.as_str().to_string()),
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
        name: &String,
    ) -> ResponseResult<()> {
        let category = Category::from_string(name).unwrap_or_default();
        update_category(
            target,
            &storage,
            &category,
            markdown_format!("🗑️ Confirm Category `{}` Removal", name),
            "🗑️ Remove",
            CommandRemoveCategory {
                name: Some(name.to_string()),
                confirm: Some(true),
            },
            Some(CommandRemoveCategory {
                name: None,
                confirm: None,
            }),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        name: &String,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_format!(
                    "❌ Category `{}` removal cancelled\\.",
                    name
                ))
                .await?;
            return Ok(());
        }

        // Parse the name string to Category
        let category = match Category::from_string(name) {
            Ok(cat) => cat,
            Err(e) => {
                target
                    .send_markdown_message(markdown_format!("❌ Invalid category name: {}", e))
                    .await?;
                return Ok(());
            }
        };

        if let Err(e) = storage.remove_category(target.chat.id, &category).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!("✅ Category `{}` removed\\.", name))
            .await?;
        Ok(())
    }
}
