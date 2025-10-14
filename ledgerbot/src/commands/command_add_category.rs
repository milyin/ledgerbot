use std::sync::Arc;

use teloxide::{prelude::Message, Bot};

use crate::{commands::command_trait::{
    CommandTrait, EmptyArg1, EmptyArg2, EmptyArg3, EmptyArg4, EmptyArg5, EmptyArg6, EmptyArg7,
    EmptyArg8, EmptyArg9,
}, storage_traits::CategoryStorageTrait};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandAddCategory {
    pub name: String,
}

impl
    CommandTrait<
        String,
        EmptyArg1<1>,
        EmptyArg2<1>,
        EmptyArg3<1>,
        EmptyArg4<1>,
        EmptyArg5<1>,
        EmptyArg6<1>,
        EmptyArg7<1>,
        EmptyArg8<1>,
        EmptyArg9<1>,
    > for CommandAddCategory
{
    type Context = Arc<dyn CategoryStorageTrait>;
    
    const NAME: &'static str = "add_category";

    fn from_arguments(
        a: String,
        _: EmptyArg1<1>,
        _: EmptyArg2<1>,
        _: EmptyArg3<1>,
        _: EmptyArg4<1>,
        _: EmptyArg5<1>,
        _: EmptyArg6<1>,
        _: EmptyArg7<1>,
        _: EmptyArg8<1>,
        _: EmptyArg9<1>,
    ) -> Self {
        CommandAddCategory { name: a }
    }
    
    fn run(bot: Bot, msg: Message, context: Self::Context) -> teloxide::prelude::ResponseResult<()> {
        todo!()
    }

    
}

impl From<CommandAddCategory> for crate::commands::Command {
    fn from(cmd: CommandAddCategory) -> Self {
        crate::commands::Command::AddCategory(cmd)
    }
}

impl From<CommandAddCategory> for String {
    fn from(cmd: CommandAddCategory) -> Self {
        format!("{} {}", CommandAddCategory::NAME, cmd.name)
    }
}