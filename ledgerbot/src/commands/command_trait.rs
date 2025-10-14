use std::str::FromStr;

use teloxide::{Bot, prelude::ResponseResult, types::Message, utils::command::ParseError};
pub trait ArgFromStr {
    fn arg_from_str(s: &str) -> Result<Option<Self>, ParseError>
    where
        Self: Sized;
}

impl<T> ArgFromStr for T
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn arg_from_str(s: &str) -> Result<Option<Self>, ParseError>
    where
        Self: Sized,
    {
        if s.is_empty() {
            return Ok(None);
        }
        s.parse::<T>()
            .map(Some)
            .map_err(|e| ParseError::Custom(Box::new(e)))
    }
}

macro_rules! impl_empty_arg_fromstr {
    ($($name:ident, $idx:expr);*) => {
        $(
            #[allow(dead_code)]
            #[derive(Default)]
            pub struct $name<const EXPECTED: usize>;

            impl<const EXPECTED: usize> ArgFromStr for $name<EXPECTED> {
                fn arg_from_str(s: &str) -> Result<Option<Self>, ParseError>
                where
                    Self: Sized,
                {
                    if s.is_empty() {
                        Ok(None)
                    } else {
                        Err(ParseError::TooManyArguments {
                            expected: EXPECTED,
                            found: 1,
                            message: format!(
                                "Expected at most {} arguments, found {}",
                                EXPECTED, 1
                            ),
                        })
                    }
                }
            }
        )*
    };
}

impl_empty_arg_fromstr!(
    EmptyArg0, 0;
    EmptyArg1, 1;
    EmptyArg2, 2;
    EmptyArg3, 3;
    EmptyArg4, 4;
    EmptyArg5, 5;
    EmptyArg6, 6;
    EmptyArg7, 7;
    EmptyArg8, 8;
    EmptyArg9, 9
);

fn get<A>(args: &[String], pos: usize) -> Result<Option<A>, ParseError>
where
    A: ArgFromStr + Default,
{
    let arg = args.get(pos).map(|s| s.as_str()).unwrap_or("");
    ArgFromStr::arg_from_str(arg)
}

fn split(arg: &str) -> Vec<String> {
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

pub trait CommandTrait: Sized {
    type A: ArgFromStr + Default;
    type B: ArgFromStr + Default;
    type C: ArgFromStr + Default;
    type D: ArgFromStr + Default;
    type E: ArgFromStr + Default;
    type F: ArgFromStr + Default;
    type G: ArgFromStr + Default;
    type H: ArgFromStr + Default;
    type I: ArgFromStr + Default;
    type J: ArgFromStr + Default;

    type Context;

    const NAME: &'static str;

    fn parse_arguments(args: String) -> Result<(Self,), ParseError> {
        let args = split(&args);
        let a = get::<Self::A>(&args, 0)?;
        let b = get::<Self::B>(&args, 1)?;
        let c = get::<Self::C>(&args, 2)?;
        let d = get::<Self::D>(&args, 3)?;
        let e = get::<Self::E>(&args, 4)?;
        let f = get::<Self::F>(&args, 5)?;
        let g = get::<Self::G>(&args, 6)?;
        let h = get::<Self::H>(&args, 7)?;
        let i = get::<Self::I>(&args, 8)?;
        let j = get::<Self::J>(&args, 9)?;
        Ok((Self::from_arguments(a, b, c, d, e, f, g, h, i, j),))
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
        j: Option<Self::J>,
    ) -> Self;

    fn run(bot: Bot, msg: Message, context: Self::Context) -> ResponseResult<()>;
}
