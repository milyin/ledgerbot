//! MarkdownString type for safe Telegram MarkdownV2 messages

use std::fmt;

/// A wrapper around String that ensures safe MarkdownV2 formatting for Telegram messages.
/// 
/// This struct can be constructed in two ways:
/// 1. `markdown!` macro - statically validates the format string at compile time
/// 2. `escape` constructor - automatically escapes markdown characters in the input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownString(String);

impl MarkdownString {
    /// Creates a MarkdownString by escaping all markdown special characters in the input.
    /// This is safe to use with any string content as all special characters will be escaped.
    /// 
    /// # Example
    /// ```rust
    /// let markdown = MarkdownString::escape("Hello! This has special chars: *bold* _italic_");
    /// // Result: "Hello\\! This has special chars: \\*bold\\* \\_italic\\_"
    /// ```
    pub fn escape<T: Into<String>>(input: T) -> Self {
        let input_string = input.into();
        let escaped = teloxide::utils::markdown::escape(&input_string);
        MarkdownString(escaped)
    }

    /// Returns the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the MarkdownString and returns the inner String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for MarkdownString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for MarkdownString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<MarkdownString> for String {
    fn from(markdown: MarkdownString) -> String {
        markdown.0
    }
}

/// Creates a MarkdownString with compile-time validation of the format string.
/// 
/// This macro validates that the format string is valid MarkdownV2 syntax at compile time,
/// ensuring balanced formatting characters and properly escaped special characters.
/// 
/// # Example
/// ```rust
/// let name = "John";
/// let markdown = markdown!("Hello *{}*\\!", name);
/// ```
/// 
/// # Compile-time errors
/// The following would cause compile-time errors:
/// ```compile_fail
/// markdown!("*unmatched bold"); // Unmatched asterisk
/// markdown!("Hello!"); // Unescaped exclamation mark
/// ```
#[macro_export]
macro_rules! markdown {
    ($format_str:expr $(, $arg:expr)*) => {{
        // Compile-time validation for Telegram MarkdownV2 format compatibility
        const _: () = $crate::macros::validate_markdownv2_format($format_str);
        
        // Escape all arguments and format the message
        let formatted_message = format!($format_str $(, teloxide::utils::markdown::escape(&$arg.to_string()))*);
        
        $crate::markdown_string::MarkdownString(formatted_message)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_constructor() {
        // Test basic escaping
        let markdown = MarkdownString::escape("Hello world");
        assert_eq!(markdown.as_str(), "Hello world");

        // Test escaping special characters
        let markdown = MarkdownString::escape("Hello! *bold* _italic_ `code`");
        assert_eq!(markdown.as_str(), "Hello\\! \\*bold\\* \\_italic\\_ \\`code\\`");

        // Test escaping all reserved characters
        let markdown = MarkdownString::escape("!.-+=>#{}[]()~|");
        assert_eq!(markdown.as_str(), "\\!\\.\\-\\+\\=\\>\\#\\{\\}\\[\\]\\(\\)\\~\\|");
    }

    #[test]
    fn test_escape_with_different_input_types() {
        // Test with &str
        let markdown = MarkdownString::escape("test");
        assert_eq!(markdown.as_str(), "test");

        // Test with String
        let markdown = MarkdownString::escape("test".to_string());
        assert_eq!(markdown.as_str(), "test");

        // Test with number (implements Into<String> via Display)
        let markdown = MarkdownString::escape(42.to_string());
        assert_eq!(markdown.as_str(), "42");
    }

    #[test]
    fn test_markdown_macro_basic() {
        let markdown = markdown!("Hello *world*\\!");
        assert_eq!(markdown.as_str(), "Hello *world*\\!");
    }

    #[test]
    fn test_markdown_macro_with_arguments() {
        let name = "John";
        let markdown = markdown!("Hello *{}*\\!", name);
        assert_eq!(markdown.as_str(), "Hello *John*\\!");
    }

    #[test]
    fn test_markdown_macro_with_special_chars_in_args() {
        let text = "special! chars* here_";
        let markdown = markdown!("Message: `{}`", text);
        assert_eq!(markdown.as_str(), "Message: `special\\! chars\\* here\\_`");
    }

    #[test]
    fn test_markdown_macro_complex() {
        let user = "Alice";
        let amount = 100;
        let category = "food*";
        let markdown = markdown!(
            "*User:* {} \n*Amount:* {} \n*Category:* `{}`", 
            user, 
            amount, 
            category
        );
        assert_eq!(
            markdown.as_str(), 
            "*User:* Alice \n*Amount:* 100 \n*Category:* `food\\*`"
        );
    }

    #[test]
    fn test_display_trait() {
        let markdown = MarkdownString::escape("Hello!");
        assert_eq!(format!("{}", markdown), "Hello\\!");
    }

    #[test]
    fn test_as_ref_trait() {
        let markdown = MarkdownString::escape("Hello!");
        let s: &str = markdown.as_ref();
        assert_eq!(s, "Hello\\!");
    }

    #[test]
    fn test_into_string() {
        let markdown = MarkdownString::escape("Hello!");
        let s: String = markdown.into_string();
        assert_eq!(s, "Hello\\!");
    }

    #[test]
    fn test_from_trait() {
        let markdown = MarkdownString::escape("Hello!");
        let s: String = markdown.into();
        assert_eq!(s, "Hello\\!");
    }

    #[test]
    fn test_clone_and_eq() {
        let markdown1 = MarkdownString::escape("Hello!");
        let markdown2 = markdown1.clone();
        assert_eq!(markdown1, markdown2);
    }

    #[test]
    fn test_real_world_usage() {
        // Simulate a real expense notification
        let user = "Alice";
        let amount = "50.00";
        let category = "groceries & food!";
        let date = "2024-12-10";
        
        // Using escape for user input (safe for any content)
        let escaped_description = MarkdownString::escape(format!("User {} spent ${} on {}", user, amount, category));
        assert_eq!(escaped_description.as_str(), "User Alice spent $50\\.00 on groceries & food\\!");
        
        // Using markdown macro for formatted messages (compile-time validated)
        let formatted_notification = markdown!(
            "💰 *Expense Added*\n\n*User:* {}\n*Amount:* ${}\n*Category:* `{}`\n*Date:* {}",
            user, amount, category, date
        );
        
        let expected = "💰 *Expense Added*\n\n*User:* Alice\n*Amount:* $50\\.00\n*Category:* `groceries & food\\!`\n*Date:* 2024\\-12\\-10";
        assert_eq!(formatted_notification.as_str(), expected);
    }

    // The following tests verify that the markdown! macro would catch invalid syntax
    // at compile time. These are included as documentation but commented out since
    // they would actually fail compilation.
    
    /*
    #[test]
    fn test_compile_time_validation_unmatched_bold() {
        // This would fail at compile time:
        // let markdown = markdown!("*unmatched bold");
    }

    #[test] 
    fn test_compile_time_validation_unescaped_exclamation() {
        // This would fail at compile time:
        // let markdown = markdown!("Hello!");
    }

    #[test]
    fn test_compile_time_validation_unmatched_italic() {
        // This would fail at compile time:
        // let markdown = markdown!("_unmatched italic");
    }

    #[test]
    fn test_compile_time_validation_unmatched_code() {
        // This would fail at compile time:
        // let markdown = markdown!("`unmatched code");
    }
    */
}