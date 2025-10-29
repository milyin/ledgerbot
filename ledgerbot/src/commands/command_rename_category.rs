use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{ menus::{select_category::select_category, update_category::update_category}, storage_traits::CategoryStorageTrait};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandRenameCategory {
    pub old_name: Option<String>,
    pub new_name: Option<String>,
}

impl CommandTrait for CommandRenameCategory {
    type A = String;
    type B = String;
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
        self.old_name.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.new_name.as_ref()
    }

    fn from_arguments(
        old_name: Option<Self::A>,
        new_name: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandRenameCategory { old_name, new_name }
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
            |name| CommandRenameCategory {
                old_name: Some(name.to_string()),
                new_name: None,
            },
            None::<NoopCommand>,
        ).await?;

        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        old_name: &String,
    ) -> ResponseResult<()> {
        update_category(
            target,
            &storage,
            old_name,
            markdown_format!("✏️ Renaming category `{}`", old_name),
            "✏️ Rename",
            CommandRenameCategory {
                old_name: Some(old_name.to_string()),
                new_name: None,
            },
            Some(CommandRenameCategory {
                old_name: None,
                new_name: None,
            })
        ).await?;

        Ok(())
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        old_name: &String,
        new_name: &String,
    ) -> ResponseResult<()> {

        if let Err(e) = storage.rename_category(target.chat.id, old_name, new_name).await {
            target.send_markdown_message(e).await?;
        }
        target
            .send_markdown_message(markdown_format!("✅ Category `{}` renamed to `{}`\\.", old_name, new_name))
            .await?;
       Ok(())
    }
}

impl From<CommandRenameCategory> for crate::commands::Command {
    fn from(cmd: CommandRenameCategory) -> Self {
        crate::commands::Command::RenameCategory(cmd)
    }
}
