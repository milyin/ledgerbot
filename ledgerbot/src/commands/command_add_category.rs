use crate::commands::command_trait::{
    CommandTrait, EmptyArg1, EmptyArg2, EmptyArg3, EmptyArg4, EmptyArg5, EmptyArg6, EmptyArg7,
    EmptyArg8, EmptyArg9,
};

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
}
