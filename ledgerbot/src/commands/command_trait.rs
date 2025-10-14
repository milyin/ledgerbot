use std::str::FromStr;

use teloxide::utils::command::ParseError;

use teloxide::{
    Bot, types::Message, prelude::ResponseResult,
};
pub trait ArgFromStr {
    fn arg_from_str(s: &str) -> Result<Self, ParseError>
    where
        Self: Sized;
}

impl<T> ArgFromStr for T
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn arg_from_str(s: &str) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        s.parse::<T>().map_err(|e| ParseError::Custom(Box::new(e)))
    }
}

macro_rules! impl_empty_arg_fromstr {
    ($($name:ident, $idx:expr);*) => {
        $(
            #[allow(dead_code)]
            #[derive(Default)]
            pub struct $name<const EXPECTED: usize>;

            impl<const EXPECTED: usize> ArgFromStr for $name<EXPECTED> {
                fn arg_from_str(s: &str) -> Result<Self, ParseError>
                where
                    Self: Sized,
                {
                    if s.is_empty() {
                        Ok($name)
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

fn get<A>(args: &[String], pos: usize) -> Result<A, ParseError>
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

pub trait CommandTrait: Sized
{
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
        let a: Self::A = get(&args, 0)?;
        let b: Self::B = get(&args, 1)?;
        let c: Self::C = get(&args, 2)?;
        let d: Self::D = get(&args, 3)?;
        let e: Self::E = get(&args, 4)?;
        let f: Self::F = get(&args, 5)?;
        let g: Self::G = get(&args, 6)?;
        let h: Self::H = get(&args, 7)?;
        let i: Self::I = get(&args, 8)?;
        let j: Self::J = get(&args, 9)?;
        Ok((Self::from_arguments(a, b, c, d, e, f, g, h, i, j),))
    }

    #[allow(clippy::too_many_arguments)]
    fn from_arguments(a: Self::A, b: Self::B, c: Self::C, d: Self::D, e: Self::E, f: Self::F, g: Self::G, h: Self::H, i: Self::I, j: Self::J) -> Self;

    fn run(bot: Bot, msg: Message, context: Self::Context) -> ResponseResult<()>;
}
