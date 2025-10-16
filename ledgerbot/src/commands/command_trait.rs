use std::{any::TypeId, error::Error, fmt::Display, str::FromStr};

use teloxide::{prelude::ResponseResult, types::{Chat, Message, MessageId}, utils::command::ParseError, Bot};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct EmptyArg;

pub trait ParseCommandArg {
    fn parse_command_arg(arg: &str) -> Result<Self, ParseError>
    where
        Self: Sized;
}

impl Display for EmptyArg {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl ParseCommandArg for EmptyArg {
    fn parse_command_arg(arg: &str) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        if arg.is_empty() {
            Ok(EmptyArg)
        } else {
            Err(ParseError::Custom(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Expected no argument for EmptyArg",
            ))))
        }
    }
}

impl<T> ParseCommandArg for T
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    fn parse_command_arg(arg: &str) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        arg.parse::<T>()
            .map_err(|e| ParseError::Custom(Box::new(e)))
    }
}

fn get<A>(args: &[String], pos: usize) -> Result<Option<A>, ParseError>
where
    A: ParseCommandArg,
{
    let parsed = args.get(pos).map(|s| A::parse_command_arg(s)).transpose()?;
    Ok(parsed)
}

fn split_with_screened_spaces(arg: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = arg.lines().next().unwrap_or("").chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&next_c) = chars.peek() {
                    if next_c == '\\' {
                        current.push('\\');
                        chars.next();
                    } else if next_c == ' ' {
                        current.push(' ');
                        chars.next();
                    } else {
                        current.push('\\');
                    }
                } else {
                    current.push('\\');
                }
            }
            ' ' => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn screen_spaces(s: &str) -> String {
    s.replace('\\', "\\\\").replace(' ', "\\ ")
}

pub trait CommandTrait: Sized {
    type A: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type B: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type C: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type D: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type E: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type F: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type G: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type H: ParseCommandArg + Default + Display + Send + Sync + 'static;
    type I: ParseCommandArg + Default + Display + Send + Sync + 'static;

    type Context;

    const NAME: &'static str;
    const PLACEHOLDERS: &[&'static str];

    #[allow(clippy::get_first)]
    fn parse_arguments(args: String) -> Result<(Self,), ParseError> {
        assert!(Self::PLACEHOLDERS.len() <= 9);
        assert!(
            Self::PLACEHOLDERS.get(0).is_some()
                || TypeId::of::<Self::A>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(1).is_some()
                || TypeId::of::<Self::B>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(2).is_some()
                || TypeId::of::<Self::C>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(3).is_some()
                || TypeId::of::<Self::D>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(4).is_some()
                || TypeId::of::<Self::E>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(5).is_some()
                || TypeId::of::<Self::F>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(6).is_some()
                || TypeId::of::<Self::G>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(7).is_some()
                || TypeId::of::<Self::H>() == TypeId::of::<EmptyArg>()
        );
        assert!(
            Self::PLACEHOLDERS.get(8).is_some()
                || TypeId::of::<Self::I>() == TypeId::of::<EmptyArg>()
        );

        let args = split_with_screened_spaces(&args);
        if args.len() > Self::PLACEHOLDERS.len() {
            return Err(ParseError::TooManyArguments {
                expected: Self::PLACEHOLDERS.len(),
                found: args.len(),
                message: format!(
                    "Expected at most {} arguments, found {}",
                    Self::PLACEHOLDERS.len(),
                    args.len()
                ),
            });
        }
        let a = get::<Self::A>(&args, 0)?;
        let b = get::<Self::B>(&args, 1)?;
        let c = get::<Self::C>(&args, 2)?;
        let d = get::<Self::D>(&args, 3)?;
        let e = get::<Self::E>(&args, 4)?;
        let f = get::<Self::F>(&args, 5)?;
        let g = get::<Self::G>(&args, 6)?;
        let h = get::<Self::H>(&args, 7)?;
        let i = get::<Self::I>(&args, 8)?;
        Ok((Self::from_arguments(a, b, c, d, e, f, g, h, i),))
    }

    #[allow(clippy::too_many_arguments)]
    fn from_arguments(
        a: Option<Self::A>,
        b: Option<Self::B>,
        c: Option<Self::C>,
        d: Option<Self::D>,
        e: Option<Self::E>,
        f: Option<Self::F>,
        g: Option<Self::G>,
        h: Option<Self::H>,
        i: Option<Self::I>,
    ) -> Self;

    fn param1(&self) -> Option<&Self::A> { assert!(TypeId::of::<Self::A>() == TypeId::of::<EmptyArg>()); None }
    fn param2(&self) -> Option<&Self::B> { assert!(TypeId::of::<Self::B>() == TypeId::of::<EmptyArg>()); None }
    fn param3(&self) -> Option<&Self::C> { assert!(TypeId::of::<Self::C>() == TypeId::of::<EmptyArg>()); None }
    fn param4(&self) -> Option<&Self::D> { assert!(TypeId::of::<Self::D>() == TypeId::of::<EmptyArg>()); None }
    fn param5(&self) -> Option<&Self::E> { assert!(TypeId::of::<Self::E>() == TypeId::of::<EmptyArg>()); None }
    fn param6(&self) -> Option<&Self::F> { assert!(TypeId::of::<Self::F>() == TypeId::of::<EmptyArg>()); None }
    fn param7(&self) -> Option<&Self::G> { assert!(TypeId::of::<Self::G>() == TypeId::of::<EmptyArg>()); None }
    fn param8(&self) -> Option<&Self::H> { assert!(TypeId::of::<Self::H>() == TypeId::of::<EmptyArg>()); None }
    fn param9(&self) -> Option<&Self::I> { assert!(TypeId::of::<Self::I>() == TypeId::of::<EmptyArg>()); None }

    async fn run0(&self, _bot: Bot, _chat: Chat, _msg_id: Option<MessageId>, _context: Self::Context) -> ResponseResult<()> {
       Ok(())
    }

    async fn run1(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
    ) -> ResponseResult<()> {
        Ok(())
    }

    async fn run2(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
    ) -> ResponseResult<()> {
        Ok(())
    }

    async fn run3(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
    ) -> ResponseResult<()> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run4(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
        _d: &Self::D,
    ) -> ResponseResult<()> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run5(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
        _d: &Self::D,
        _e: &Self::E,
    ) -> ResponseResult<()> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run6(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
        _d: &Self::D,
        _e: &Self::E,
        _f: &Self::F,
    ) -> ResponseResult<()> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run7(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
        _d: &Self::D,
        _e: &Self::E,
        _f: &Self::F,
        _g: &Self::G,
    ) -> ResponseResult<()> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run8(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
        _d: &Self::D,
        _e: &Self::E,
        _f: &Self::F,
        _g: &Self::G,
        _h: &Self::H,
    ) -> ResponseResult<()> {
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn run9(
        &self,
        _bot: Bot,
        _chat: Chat,
        _msg_id: Option<MessageId>,
        _context: Self::Context,
        _a: &Self::A,
        _b: &Self::B,
        _c: &Self::C,
        _d: &Self::D,
        _e: &Self::E,
        _f: &Self::F,
        _g: &Self::G,
        _h: &Self::H,
        _i: &Self::I,
    ) -> ResponseResult<()> {
        Ok(())
    }

    async fn run(&self, bot: Bot, chat: Chat, msg_id: Option<MessageId>, context: Self::Context) -> ResponseResult<()> {
        match (
            self.param1(),
            self.param2(),
            self.param3(),
            self.param4(),
            self.param5(),
            self.param6(),
            self.param7(),
            self.param8(),
            self.param9(),
        ) {
            (None, None, None, None, None, None, None, None, None) => self.run0(bot, chat, msg_id, context).await,
            (Some(a), None, None, None, None, None, None, None, None) => {
                self.run1(bot, chat, msg_id, context, a).await
            }
            (Some(a), Some(b), None, None, None, None, None, None, None) => {
                self.run2(bot, chat, msg_id, context, a, b).await
            }
            (Some(a), Some(b), Some(c), None, None, None, None, None, None) => {
                self.run3(bot, chat, msg_id, context, a, b, c).await
            }
            (Some(a), Some(b), Some(c), Some(d), None, None, None, None, None) => {
                self.run4(bot, chat, msg_id, context, a, b, c, d).await
            }
            (Some(a), Some(b), Some(c), Some(d), Some(e), None, None, None, None) => {
                self.run5(bot, chat, msg_id, context, a, b, c, d, e).await
            }
            (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), None, None, None) => {
                self.run6(bot, chat, msg_id, context, a, b, c, d, e, f).await
            }
            (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), None, None) => {
                self.run7(bot, chat, msg_id, context, a, b, c, d, e, f, g).await
            }
            (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), None) => {
                self.run8(bot, chat, msg_id, context, a, b, c, d, e, f, g, h).await
            }
            (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i)) => {
                self.run9(bot, chat, msg_id, context, a, b, c, d, e, f, g, h, i).await
            },
            _ => Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown("Internal bot error: missing middle argument. Should not happen".into())))
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn to_command_string(&self, complete: bool) -> String {
        let params: Vec<Option<String>> = vec![
            self.param1().map(|v| v.to_string()),
            self.param2().map(|v| v.to_string()),
            self.param3().map(|v| v.to_string()),
            self.param4().map(|v| v.to_string()),
            self.param5().map(|v| v.to_string()),
            self.param6().map(|v| v.to_string()),
            self.param7().map(|v| v.to_string()),
            self.param8().map(|v| v.to_string()),
            self.param9().map(|v| v.to_string()),
        ];

        let max_index = if !complete {
            (0..9).rev().find(|&i| params[i].is_some())
        } else if Self::PLACEHOLDERS.is_empty() {
            None
        } else {
            Some(Self::PLACEHOLDERS.len() - 1)
        };

        let mut command_parts = vec![format!("/{}", Self::NAME)];
        if let Some(max_i) = max_index {
            for i in 0..=max_i {
                let part = params[i]
                    .clone()
                    .unwrap_or(Self::PLACEHOLDERS[i].to_string());
                command_parts.push(screen_spaces(&part));
            }
        }
        let mut command = command_parts.join(" ");
        if command_parts.len() < Self::PLACEHOLDERS.len() + 1 {
            command.push(' ');
        }
        command
    }
}
