/// Creates a MarkdownString with compile-time validation of the format string.
#[macro_export]
macro_rules! markdown_string {
    ($format_str:expr) => {{
        // Compile-time validation for Telegram MarkdownV2 format compatibility
        const _: () = $crate::markdown::validate_markdownv2_format($format_str);
        $crate::markdown::MarkdownString::from_validated_string($format_str)
    }};
}

/// Formats a MarkdownString using either a &str literal (with compile-time validation) or a MarkdownString as the template.
///
/// If a &str literal is provided, it will be validated at compile-time using `markdown_string!`.
/// Arguments must be types that can be converted to MarkdownString.
#[macro_export]
macro_rules! markdown_format {
    // Match string literals and apply compile-time validation
    ($format_str:literal $(, $arg:expr)*) => {
        $crate::markdown_format!($crate::markdown_string!($format_str) $(, $arg)*)
    };
    // Match MarkdownString expressions
    ($format_markdown:expr $(, $arg:expr)*) => {{
        // Use the MarkdownString directly
        let markdown_string: $crate::markdown::MarkdownString = $format_markdown;

        // Get the format string from the MarkdownString
        let format_str = markdown_string.as_str();

        // Convert all arguments to strings for replacement
        let escaped_args: Vec<String> = vec![$({
            // Convert to MarkdownString for type safety
            let arg_markdown: $crate::markdown::MarkdownString = $arg.into();
            arg_markdown.as_str().to_string()
        }),*];

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
