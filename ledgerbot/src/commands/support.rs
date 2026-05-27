use std::{error::Error, fmt::Display, str::FromStr};

use telluride::utils::{screen_spaces, split_with_screened_spaces};
use teloxide::utils::command::ParseError;

pub fn split_args(args: &str, max_args: usize) -> Result<Vec<String>, ParseError> {
    let args = split_with_screened_spaces(args);
    if args.len() > max_args {
        return Err(ParseError::TooManyArguments {
            expected: max_args,
            found: args.len(),
            message: format!(
                "Expected at most {} arguments, found {}",
                max_args,
                args.len()
            ),
        });
    }
    Ok(args)
}

pub fn parse_optional<T>(args: &[String], pos: usize) -> Result<Option<T>, ParseError>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    args.get(pos)
        .map(|s| s.parse::<T>().map_err(|e| ParseError::Custom(Box::new(e))))
        .transpose()
}

pub fn build_command_string(
    name: &str,
    placeholders: &[&str],
    params: &[Option<String>],
    complete: bool,
) -> String {
    let max_index = if !complete {
        (0..params.len()).rev().find(|&i| params[i].is_some())
    } else if placeholders.is_empty() {
        None
    } else {
        Some(placeholders.len() - 1)
    };

    let mut command_parts = vec![format!("/{}", name)];
    if let Some(max_i) = max_index {
        for i in 0..=max_i {
            let part = params[i]
                .clone()
                .unwrap_or_else(|| placeholders[i].to_string());
            command_parts.push(screen_spaces(&part));
        }
    }

    let mut command = command_parts.join(" ");
    if command_parts.len() < placeholders.len() + 1 {
        command.push(' ');
    }
    command
}

pub fn display_opt<T: Display>(value: Option<&T>) -> Option<String> {
    value.map(ToString::to_string)
}

#[macro_export]
macro_rules! impl_command_io {
    ($ty:ident, $name:literal, []) => {
        impl $ty {
            pub const NAME: &'static str = $name;
            pub const PLACEHOLDERS: &'static [&'static str] = &[];

            pub fn parse_arguments(args: String) -> Result<(Self,), teloxide::utils::command::ParseError> {
                let args = $crate::commands::support::split_args(&args, 0)?;
                debug_assert!(args.is_empty());
                Ok((Self,))
            }

            pub fn to_command_string(&self, complete: bool) -> String {
                let params: Vec<Option<String>> = Vec::new();
                $crate::commands::support::build_command_string(
                    Self::NAME,
                    Self::PLACEHOLDERS,
                    &params,
                    complete,
                )
            }
        }
    };
    ($ty:ident, $name:literal, [$($placeholder:expr),+ $(,)?], $field1:ident : $field_ty1:ty) => {
        impl $ty {
            pub const NAME: &'static str = $name;
            pub const PLACEHOLDERS: &'static [&'static str] = &[$($placeholder),*];

            pub fn parse_arguments(args: String) -> Result<(Self,), teloxide::utils::command::ParseError> {
                let args = $crate::commands::support::split_args(&args, Self::PLACEHOLDERS.len())?;
                Ok((Self {
                    $field1: $crate::commands::support::parse_optional::<$field_ty1>(&args, 0)?,
                },))
            }

            pub fn to_command_string(&self, complete: bool) -> String {
                let params = vec![
                    $crate::commands::support::display_opt(self.$field1.as_ref()),
                ];
                $crate::commands::support::build_command_string(
                    Self::NAME,
                    Self::PLACEHOLDERS,
                    &params,
                    complete,
                )
            }
        }
    };
    ($ty:ident, $name:literal, [$($placeholder:expr),+ $(,)?], $field1:ident : $field_ty1:ty, $field2:ident : $field_ty2:ty) => {
        impl $ty {
            pub const NAME: &'static str = $name;
            pub const PLACEHOLDERS: &'static [&'static str] = &[$($placeholder),*];

            pub fn parse_arguments(args: String) -> Result<(Self,), teloxide::utils::command::ParseError> {
                let args = $crate::commands::support::split_args(&args, Self::PLACEHOLDERS.len())?;
                Ok((Self {
                    $field1: $crate::commands::support::parse_optional::<$field_ty1>(&args, 0)?,
                    $field2: $crate::commands::support::parse_optional::<$field_ty2>(&args, 1)?,
                },))
            }

            pub fn to_command_string(&self, complete: bool) -> String {
                let params = vec![
                    $crate::commands::support::display_opt(self.$field1.as_ref()),
                    $crate::commands::support::display_opt(self.$field2.as_ref()),
                ];
                $crate::commands::support::build_command_string(
                    Self::NAME,
                    Self::PLACEHOLDERS,
                    &params,
                    complete,
                )
            }
        }
    };
    ($ty:ident, $name:literal, [$($placeholder:expr),+ $(,)?], $field1:ident : $field_ty1:ty, $field2:ident : $field_ty2:ty, $field3:ident : $field_ty3:ty) => {
        impl $ty {
            pub const NAME: &'static str = $name;
            pub const PLACEHOLDERS: &'static [&'static str] = &[$($placeholder),*];

            pub fn parse_arguments(args: String) -> Result<(Self,), teloxide::utils::command::ParseError> {
                let args = $crate::commands::support::split_args(&args, Self::PLACEHOLDERS.len())?;
                Ok((Self {
                    $field1: $crate::commands::support::parse_optional::<$field_ty1>(&args, 0)?,
                    $field2: $crate::commands::support::parse_optional::<$field_ty2>(&args, 1)?,
                    $field3: $crate::commands::support::parse_optional::<$field_ty3>(&args, 2)?,
                },))
            }

            pub fn to_command_string(&self, complete: bool) -> String {
                let params = vec![
                    $crate::commands::support::display_opt(self.$field1.as_ref()),
                    $crate::commands::support::display_opt(self.$field2.as_ref()),
                    $crate::commands::support::display_opt(self.$field3.as_ref()),
                ];
                $crate::commands::support::build_command_string(
                    Self::NAME,
                    Self::PLACEHOLDERS,
                    &params,
                    complete,
                )
            }
        }
    };
    ($ty:ident, $name:literal, [$($placeholder:expr),+ $(,)?], $field1:ident : $field_ty1:ty, $field2:ident : $field_ty2:ty, $field3:ident : $field_ty3:ty, $field4:ident : $field_ty4:ty) => {
        impl $ty {
            pub const NAME: &'static str = $name;
            pub const PLACEHOLDERS: &'static [&'static str] = &[$($placeholder),*];

            pub fn parse_arguments(args: String) -> Result<(Self,), teloxide::utils::command::ParseError> {
                let args = $crate::commands::support::split_args(&args, Self::PLACEHOLDERS.len())?;
                Ok((Self {
                    $field1: $crate::commands::support::parse_optional::<$field_ty1>(&args, 0)?,
                    $field2: $crate::commands::support::parse_optional::<$field_ty2>(&args, 1)?,
                    $field3: $crate::commands::support::parse_optional::<$field_ty3>(&args, 2)?,
                    $field4: $crate::commands::support::parse_optional::<$field_ty4>(&args, 3)?,
                },))
            }

            pub fn to_command_string(&self, complete: bool) -> String {
                let params = vec![
                    $crate::commands::support::display_opt(self.$field1.as_ref()),
                    $crate::commands::support::display_opt(self.$field2.as_ref()),
                    $crate::commands::support::display_opt(self.$field3.as_ref()),
                    $crate::commands::support::display_opt(self.$field4.as_ref()),
                ];
                $crate::commands::support::build_command_string(
                    Self::NAME,
                    Self::PLACEHOLDERS,
                    &params,
                    complete,
                )
            }
        }
    };
}

#[macro_export]
macro_rules! impl_command_execute_0 {
    ($ty:ident, $ctx_ty:ty) => {
        impl $ty {
            pub async fn execute(
                &self,
                target: &$crate::ui::CommandContext,
                context: $ctx_ty,
            ) -> teloxide::prelude::ResponseResult<()> {
                self.run0(target, context).await
            }
        }
    };
}

#[macro_export]
macro_rules! impl_command_execute_1 {
    ($ty:ident, $ctx_ty:ty, $a:ident) => {
        impl $ty {
            pub async fn execute(
                &self,
                target: &$crate::ui::CommandContext,
                context: $ctx_ty,
            ) -> teloxide::prelude::ResponseResult<()> {
                match self.$a.as_ref() {
                    None => self.run0(target, context).await,
                    Some(a) => self.run1(target, context, a).await,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_command_execute_2 {
    ($ty:ident, $ctx_ty:ty, $a:ident, $b:ident) => {
        impl $ty {
            pub async fn execute(
                &self,
                target: &$crate::ui::CommandContext,
                context: $ctx_ty,
            ) -> teloxide::prelude::ResponseResult<()> {
                match (self.$a.as_ref(), self.$b.as_ref()) {
                    (None, None) => self.run0(target, context).await,
                    (Some(a), None) => self.run1(target, context, a).await,
                    (Some(a), Some(b)) => self.run2(target, context, a, b).await,
                    _ => Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(
                        "Internal bot error: missing middle argument. Should not happen".into(),
                    ))),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_command_execute_3 {
    ($ty:ident, $ctx_ty:ty, $a:ident, $b:ident, $c:ident) => {
        impl $ty {
            pub async fn execute(
                &self,
                target: &$crate::ui::CommandContext,
                context: $ctx_ty,
            ) -> teloxide::prelude::ResponseResult<()> {
                match (self.$a.as_ref(), self.$b.as_ref(), self.$c.as_ref()) {
                    (None, None, None) => self.run0(target, context).await,
                    (Some(a), None, None) => self.run1(target, context, a).await,
                    (Some(a), Some(b), None) => self.run2(target, context, a, b).await,
                    (Some(a), Some(b), Some(c)) => self.run3(target, context, a, b, c).await,
                    _ => Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(
                        "Internal bot error: missing middle argument. Should not happen".into(),
                    ))),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_command_execute_4 {
    ($ty:ident, $ctx_ty:ty, $a:ident, $b:ident, $c:ident, $d:ident) => {
        impl $ty {
            pub async fn execute(
                &self,
                target: &$crate::ui::CommandContext,
                context: $ctx_ty,
            ) -> teloxide::prelude::ResponseResult<()> {
                match (
                    self.$a.as_ref(),
                    self.$b.as_ref(),
                    self.$c.as_ref(),
                    self.$d.as_ref(),
                ) {
                    (None, None, None, None) => self.run0(target, context).await,
                    (Some(a), None, None, None) => self.run1(target, context, a).await,
                    (Some(a), Some(b), None, None) => self.run2(target, context, a, b).await,
                    (Some(a), Some(b), Some(c), None) => self.run3(target, context, a, b, c).await,
                    (Some(a), Some(b), Some(c), Some(d)) => {
                        self.run4(target, context, a, b, c, d).await
                    }
                    _ => Err(teloxide::RequestError::Api(teloxide::ApiError::Unknown(
                        "Internal bot error: missing middle argument. Should not happen".into(),
                    ))),
                }
            }
        }
    };
}
