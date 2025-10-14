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

impl CommandTrait for CommandAddCategory
{
    type A = String;
    type B = EmptyArg1<1>;
    type C = EmptyArg2<1>;
    type D = EmptyArg3<1>;
    type E = EmptyArg4<1>;
    type F = EmptyArg5<1>;
    type G = EmptyArg6<1>;
    type H = EmptyArg7<1>;
    type I = EmptyArg8<1>;
    type J = EmptyArg9<1>;

    type Context = Arc<dyn CategoryStorageTrait>;
    
    const NAME: &'static str = "add_category";

    fn from_arguments(
        a: Self::A,
        _: Self::B,
        _: Self::C,
        _: Self::D,
        _: Self::E,
        _: Self::F,
        _: Self::G,
        _: Self::H,
        _: Self::I,
        _: Self::J,
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