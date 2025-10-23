use std::sync::Arc;

use teloxide::{prelude::ResponseResult, utils::markdown};
use yoroolbot::{markdown_format, markdown_string};

use crate::{commands::command_trait::{CommandReplyTarget, CommandTrait, EmptyArg}, storage_traits::CategoryStorageTrait};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandAddFilter {
    pub category: Option<String>,
    pub pattern: Option<String>,
}

impl CommandTrait for CommandAddFilter {
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
        CommandAddFilter {
            category,
            pattern,
        }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        Ok(())
    }
    
}