/// Creates a MarkdownString with compile-time validation of the format string.
#[macro_export]
macro_rules! markdown_string {
    ($format_str:expr) => {{
        // Compile-time validation for Telegram MarkdownV2 format compatibility
        const _: () = $crate::markdown::validate_markdownv2_format($format_str);
        $crate::markdown::MarkdownString::from_validated_string($format_str)
    }};
}

/// Helper macro to process arguments in any order, handling @raw and regular arguments.
///
/// This uses incremental TT munching to process one argument at a time.
#[doc(hidden)]
#[macro_export]
macro_rules! md_process_args {
    // Base case: no more arguments, return accumulated vector
    (@munch [] -> [$($processed:tt)*]) => {
        vec![$($processed)*]
    };

    // Process @raw argument
    (@munch [@raw $raw_arg:expr $(, $($tail:tt)*)?] -> [$($processed:tt)*]) => {
        $crate::md_process_args!(@munch [$($($tail)*)?] -> [
            $($processed)*
            {
                let markdown: $crate::markdown::MarkdownString = $raw_arg;
                markdown.as_str().to_string()
            },
        ])
    };

    // Process regular argument
    (@munch [$arg:expr $(, $($tail:tt)*)?] -> [$($processed:tt)*]) => {
        $crate::md_process_args!(@munch [$($($tail)*)?] -> [
            $($processed)*
            {
                let arg_markdown: $crate::markdown::MarkdownString = $arg.into();
                arg_markdown.as_str().to_string()
            },
        ])
    };

    // Entry point
    ($($args:tt)*) => {
        $crate::md_process_args!(@munch [$($args)*] -> [])
    };
}

/// Formats a MarkdownString using either a &str literal (with compile-time validation) or a MarkdownString as the template.
///
/// If a &str literal is provided, it will be validated at compile-time using `markdown_string!`.
/// Arguments must be types that implement `Into<MarkdownString>`.
///
/// To pass a MarkdownString without re-escaping (for pre-formatted markdown), prefix it with `@raw`.
/// You can mix `@raw` and regular arguments in any order.
///
/// # Examples
/// ```ignore
/// let formatted = markdown_string!("*bold*");
/// let result = markdown_format!("Value: {}, Header: {}, Plain: {}", "text", @raw formatted, "more");
/// ```
#[macro_export]
macro_rules! markdown_format {
    // String literal with no arguments
    ($format_str:literal) => {
        $crate::markdown_string!($format_str)
    };

    // String literal with arguments - delegate to MarkdownString version
    ($format_str:literal, $($args:tt)*) => {
        $crate::markdown_format!($crate::markdown_string!($format_str), $($args)*)
    };

    // MarkdownString with no arguments
    ($format_markdown:expr) => {{
        let markdown_string: $crate::markdown::MarkdownString = $format_markdown;
        markdown_string
    }};

    // MarkdownString with arguments
    ($format_markdown:expr, $($args:tt)*) => {{
        let markdown_string: $crate::markdown::MarkdownString = $format_markdown;
        let format_str = markdown_string.as_str();

        // Process all arguments using the helper macro
        let escaped_args: Vec<String> = $crate::md_process_args!($($args)*);

        // Replace placeholders with converted arguments
        let mut result = format_str.to_string();
        for escaped_arg in escaped_args {
            if let Some(placeholder_pos) = result.find("{}") {
                result.replace_range(placeholder_pos..placeholder_pos + 2, &escaped_arg);
            }
        }

        $crate::markdown::MarkdownString::from_validated_string(result)
    }};
}
