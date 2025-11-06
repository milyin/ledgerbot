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
pub struct CommandRenameCategory {
    pub old_category: Option<Category>,
    pub new_category: Option<Category>,
}

impl CommandTrait for CommandRenameCategory {
    type A = Category;
    type B = Category;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "rename_category";
    const PLACEHOLDERS: &[&'static str] = &["<old_name>", "<new_name>"];

    fn param1(&self) -> Option<&Self::A> {
        self.old_category.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.new_category.as_ref()
    }

    fn from_arguments(
        old_category: Option<Self::A>,
        new_category: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandRenameCategory {
            old_category,
            new_category,
        }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("✏️ Select Category to rename"),
            |category| CommandRenameCategory {
                old_category: Some(category.clone()),
                new_category: None,
            },
            None::<NoopCommand>,
        )
        .await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        old_category: &Category,
    ) -> ResponseResult<()> {
        update_category(
            target,
            &storage,
            old_category,
            markdown_format!("✏️ Renaming category `{}`", old_category.as_str()),
            "✏️ Rename",
            CommandRenameCategory {
                old_category: Some(old_category.clone()),
                new_category: None,
            },
            Some(CommandRenameCategory {
                old_category: None,
                new_category: None,
            }),
        )
        .await?;

        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        old_category: &Category,
        new_category: &Category,
    ) -> ResponseResult<()> {
        if let Err(e) = storage
            .rename_category(target.chat.id, old_category, new_category)
            .await
        {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!(
                "✅ Category `{}` renamed to `{}`\\.",
                old_category.as_str(),
                new_category.as_str()
            ))
            .await?;
        Ok(())
    }
}

impl From<CommandRenameCategory> for crate::commands::Command {
    fn from(cmd: CommandRenameCategory) -> Self {
        crate::commands::Command::RenameCategory(cmd)
    }
}
