use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::markdown_string;

use crate::{
    commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    menus::select_category::select_category,
    storage_traits::CategoryStorageTrait,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddFilter2 {
    pub category: Option<String>,
}

impl CommandTrait for CommandAddFilter2 {
    type A = String;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "add_filter2";
    const PLACEHOLDERS: &[&'static str] = &["<category>"];

    fn from_arguments(
        category: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandAddFilter2 { category }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage,
            markdown_string!("➕ Select Category to add filter"),
            |name| CommandAddFilter2 {
                category: Some(name.to_string()),
            },
            None::<NoopCommand>,
        )
        .await
    }
}

impl From<CommandAddFilter2> for crate::commands::Command {
    fn from(cmd: CommandAddFilter2) -> Self {
        crate::commands::Command::AddFilter2(cmd)
    }
}
