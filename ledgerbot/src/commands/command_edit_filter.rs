use std::sync::Arc;

use teloxide::{prelude::{Message, ResponseResult}, Bot};

use crate::{commands::command_trait::{CommandTrait, EmptyArg}, storage_traits::CategoryStorageTrait};

pub struct CommandEditFilter {
    category: Option<String>,
    position: Option<usize>,
    pattern: Option<String>,
}

impl CommandTrait for CommandEditFilter {
    type A = String;
    type B = usize;
    type C = String;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;
    type J = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "edit_filter";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<position>", "<new_pattern>"];

    fn from_arguments(
        a: Option<Self::A>,
        b: Option<Self::B>,
        c: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
        _: Option<Self::J>,
    ) -> Self {
        CommandEditFilter {
            category: a,
            position: b,
            pattern: c,
        }
    }

    fn param0(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }
    fn param1(&self) -> Option<&Self::B> {
        self.position.as_ref()
    }
    fn param2(&self) -> Option<&Self::C> {
        self.pattern.as_ref()
    }
    fn param3(&self) -> Option<&Self::D> {
        None
    }
    fn param4(&self) -> Option<&Self::E> {
        None
    }
    fn param5(&self) -> Option<&Self::F> {
        None
    }
    fn param6(&self) -> Option<&Self::G> {
        None
    }
    fn param7(&self) -> Option<&Self::H> {
        None
    }
    fn param8(&self) -> Option<&Self::I> {
        None
    }
    fn param9(&self) -> Option<&Self::J> {
        None
    }
    
    async fn run(&self, bot: Bot, msg: Message, context: Self::Context) -> ResponseResult<()> {
        let chat_id = msg.chat.id;
    }


}