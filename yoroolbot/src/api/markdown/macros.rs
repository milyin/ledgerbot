
/// Creates a MarkdownString with compile-time validation of the format string.
#[macro_export]
macro_rules! markdown {
    ($format_str:expr $(, $arg:expr)*) => {{
        // Compile-time validation for Telegram MarkdownV2 format compatibility
        const _: () = $crate::api::markdown::validate::validate_markdownv2_format($format_str);

        // Escape all arguments and format the message
        let formatted_message = format!($format_str $(, teloxide::utils::markdown::escape(&$arg.to_string()))*);

        $crate::api::markdown::string::MarkdownString::from_validated_string(formatted_message)
    }};
}

/// Formats a MarkdownString using any type that can be converted to MarkdownString as the format template.
#[macro_export]
macro_rules! markdown_format {
    ($format_markdown:expr $(, $arg:expr)*) => {{
        // Convert the input to MarkdownString using Into trait
        let markdown_string: $crate::api::markdown::string::MarkdownString = $format_markdown.into();

        // Get the format string from the MarkdownString
        let format_str = markdown_string.as_str();

        // Convert all arguments to strings for replacement
        let escaped_args: Vec<String> = vec![$({
            // Try to convert to MarkdownString first for type safety
            let arg_markdown: $crate::api::markdown::string::MarkdownString = $arg.into();
            arg_markdown.as_str().to_string()
        }),*];

        // Replace placeholders with converted arguments
        let mut result = format_str.to_string();
        for escaped_arg in escaped_args {
            if let Some(placeholder_pos) = result.find("{}") {
                result.replace_range(placeholder_pos..placeholder_pos + 2, &escaped_arg);
            }
        }

        $crate::api::markdown::string::MarkdownString::from_validated_string(result)
    }};
}
