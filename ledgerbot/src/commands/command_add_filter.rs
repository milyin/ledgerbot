use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_format,
};

use crate::{
    commands::command_add_words_filter::CommandAddWordsFilter,
    storages::{Category, Storage},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandAddFilter {
    pub category: Option<Category>,
    pub pattern: Option<String>,
}

impl CommandTrait for CommandAddFilter {
    type A = Category;
    type B = String;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<Storage>;

    const NAME: &'static str = "add_filter";

    const PLACEHOLDERS: &[&'static str] = &["<category>", "<pattern>"];

    fn from_arguments(
        category: Option<Self::A>,
        pattern: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddFilter { category, pattern }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.pattern.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        CommandAddWordsFilter::default().run(target, storage).await
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &Category,
    ) -> ResponseResult<()> {
        CommandAddWordsFilter {
            category: Some(category.clone()),
            page: None,
            words: None,
        }
        .run(target, storage)
        .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &Category,
        pattern: &String,
    ) -> ResponseResult<()> {
        if let Err(msg) = storage.categories()
            .add_category_filter(category, pattern.clone())
            .await
        {
            target.send_markdown_message(msg).await?;
            return Ok(());
        };
        target
            .send_markdown_message(markdown_format!(
                "✅ Filter `{}` added to category `{}`\\.",
                pattern,
                category.as_str()
            ))
            .await?;
        Ok(())
    }
}
