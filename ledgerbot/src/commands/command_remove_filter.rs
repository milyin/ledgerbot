use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{markdown_format, markdown_string};

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    menus::{
        common::read_category_filter_by_index, select_category::select_category,
        select_category_filter::select_category_filter,
    },
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandRemoveFilter {
    pub category: Option<String>,
    pub position: Option<usize>,
}

impl CommandRemoveFilter {
    pub fn new(category: Option<String>, position: Option<usize>) -> Self {
        Self { category, position }
    }
}

impl CommandTrait for CommandRemoveFilter {
    type A = String;
    type B = usize;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "remove_filter";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<position>"];

    fn from_arguments(
        a: Option<Self::A>,
        b: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandRemoveFilter {
            category: a,
            position: b,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }
    fn param2(&self) -> Option<&Self::B> {
        self.position.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("🗑️ Select Category for removing filter"),
            |name| CommandRemoveFilter {
                category: Some(name.to_string()),
                position: None,
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
            markdown_format!("🗑️ Select Filter to remove from category `{}`", name),
            |idx| CommandRemoveFilter {
                category: Some(name.clone()),
                position: Some(idx),
            },
            Some(CommandRemoveFilter::default()),
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
        let Some(pattern) = read_category_filter_by_index(
            target,
            &storage,
            name,
            *idx,
            Some(CommandRemoveFilter {
                category: Some(name.clone()),
                position: None,
            }),
        )
        .await?
        else {
            return Ok(());
        };

        // Remove the filter
        storage
            .remove_category_filter(target.chat.id, name, &pattern)
            .await;

        target
            .send_markdown_message(markdown_format!(
                "✅ Filter **\\#{}** \\(`{}`\\) removed from category `{}`\\.",
                *idx,
                pattern,
                name
            ))
            .await?;

        Ok(())
    }
}

impl From<CommandRemoveFilter> for crate::commands::Command {
    fn from(cmd: CommandRemoveFilter) -> Self {
        crate::commands::Command::RemoveFilter(cmd)
    }
}
