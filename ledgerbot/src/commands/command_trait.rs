use std::str::FromStr;

use teloxide::utils::command::ParseError;

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
where A: ArgFromStr + Default,
{
    let arg = args.get(pos)
        .map(|s| s.as_str())
        .unwrap_or("");
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

pub trait CommandTrait<A, B, C, D, E, F, G, H, I, J>
where
    A: ArgFromStr + Default,
    B: ArgFromStr + Default,
    C: ArgFromStr + Default,
    D: ArgFromStr + Default,
    E: ArgFromStr + Default,
    F: ArgFromStr + Default,
    G: ArgFromStr + Default,
    H: ArgFromStr + Default,
    I: ArgFromStr + Default,
    J: ArgFromStr + Default,
    Self: Sized,
{
    fn parse_arguments(args: String) -> Result<(Self,), ParseError> {
        let args = split(&args);
        let a: A = get(&args, 0)?;
        let b: B = get(&args, 1)?;
        let c: C = get(&args, 2)?;
        let d: D = get(&args, 3)?;
        let e: E = get(&args, 4)?;
        let f: F = get(&args, 5)?;
        let g: G = get(&args, 6)?;
        let h: H = get(&args, 7)?;
        let i: I = get(&args, 8)?;
        let j: J = get(&args, 9)?;
        Ok((Self::from_arguments(a, b, c, d, e, f, g, h, i, j),))
    }

    #[allow(clippy::too_many_arguments)]
    fn from_arguments(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J) -> Self;
}
