//! MarkdownString type for safe Telegram MarkdownV2 messages

use std::{fmt, ops::Add};

use teloxide::{
    payloads::{EditMessageTextSetters, SendMessage, SendMessageSetters},
    prelude::{Requester, ResponseResult},
    types::{
        Message, MessageId,
        ParseMode::{self, MarkdownV2},
        Recipient,
    },
    Bot,
};

/// A wrapper around String that ensures safe MarkdownV2 formatting for Telegram messages.
///
/// This struct can only be constructed through safe methods:
/// 1. `markdown_string!` macro - statically validates the format string at compile time
/// 2. `escape` constructor - automatically escapes markdown characters in the input
/// 3. `new` constructor - creates an empty MarkdownString
/// 4. `From`/`Into` trait - automatically escapes the input for safety
///
/// Direct construction is not allowed to ensure all content is either validated or escaped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownString(String);

impl MarkdownString {
    /// Creates a MarkdownString by escaping all markdown special characters in the input.
    /// This is safe to use with any string content as all special characters will be escaped.
    ///
    /// # Example
    /// ```rust
    /// use yoroolbot::markdown::MarkdownString;
    ///
    /// let markdown = MarkdownString::escape("Hello! This has special chars: *bold* _italic_");
    /// // Result: "Hello\\! This has special chars: \\*bold\\* \\_italic\\_"
    /// ```
    pub fn escape<T: Into<String>>(input: T) -> Self {
        let input_string = input.into();
        let escaped = teloxide::utils::markdown::escape(&input_string);
        MarkdownString(escaped)
    }

    /// Creates an empty MarkdownString.
    /// This is equivalent to `MarkdownString::escape("")` but more idiomatic.
    ///
    /// # Example
    /// ```rust
    /// use yoroolbot::markdown::MarkdownString;
    ///
    /// let markdown = MarkdownString::new();
    /// assert_eq!(markdown.as_str(), "");
    /// ```
    pub fn new() -> Self {
        MarkdownString(String::new())
    }

    /// Private constructor for use by the markdown_string! macro after compile-time validation.
    /// This should only be called by trusted code that has already validated the input.
    #[doc(hidden)]
    pub fn from_validated_string(s: impl Into<String>) -> Self {
        MarkdownString(s.into())
    }

    /// Test-only constructor for creating templates in tests.
    /// This bypasses safety checks and should only be used in tests.
    #[cfg(test)]
    pub(crate) fn test_template(s: &str) -> Self {
        MarkdownString(s.to_string())
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

impl Default for MarkdownString {
    /// Creates an empty MarkdownString using `MarkdownString::new()`.
    fn default() -> Self {
        Self::new()
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

impl From<String> for MarkdownString {
    fn from(s: String) -> Self {
        MarkdownString::escape(s)
    }
}

impl From<&String> for MarkdownString {
    fn from(s: &String) -> Self {
        MarkdownString::escape(s)
    }
}

impl From<&str> for MarkdownString {
    fn from(s: &str) -> Self {
        MarkdownString::escape(s)
    }
}

// Implement From for common numeric types
impl From<i32> for MarkdownString {
    fn from(n: i32) -> Self {
        MarkdownString::escape(n.to_string())
    }
}

impl From<i64> for MarkdownString {
    fn from(n: i64) -> Self {
        MarkdownString::escape(n.to_string())
    }
}

impl From<f32> for MarkdownString {
    fn from(n: f32) -> Self {
        MarkdownString::escape(n.to_string())
    }
}

impl From<f64> for MarkdownString {
    fn from(n: f64) -> Self {
        MarkdownString::escape(n.to_string())
    }
}

impl From<usize> for MarkdownString {
    fn from(n: usize) -> Self {
        MarkdownString::escape(n.to_string())
    }
}

impl From<isize> for MarkdownString {
    fn from(n: isize) -> Self {
        MarkdownString::escape(n.to_string())
    }
}

// Implement Add operation for concatenating MarkdownStrings
impl Add for MarkdownString {
    type Output = MarkdownString;

    fn add(self, other: MarkdownString) -> MarkdownString {
        let combined = format!("{}{}", self.0, other.0);
        MarkdownString::from_validated_string(combined)
    }
}

impl Add<&MarkdownString> for MarkdownString {
    type Output = MarkdownString;

    fn add(self, other: &MarkdownString) -> MarkdownString {
        let combined = format!("{}{}", self.0, other.0);
        MarkdownString::from_validated_string(combined)
    }
}

impl Add<MarkdownString> for &MarkdownString {
    type Output = MarkdownString;

    fn add(self, other: MarkdownString) -> MarkdownString {
        let combined = format!("{}{}", self.0, other.0);
        MarkdownString::from_validated_string(combined)
    }
}

impl Add<&MarkdownString> for &MarkdownString {
    type Output = MarkdownString;

    fn add(self, other: &MarkdownString) -> MarkdownString {
        let combined = format!("{}{}", self.0, other.0);
        MarkdownString::from_validated_string(combined)
    }
}

/// Trait for sending markdown messages with Bot
///
/// This trait provides a convenient method for sending MarkdownString messages
/// using teloxide Bot, automatically setting the parse mode to MarkdownV2.
///
/// # Example
///
/// ```rust
/// use yoroolbot::markdown::{MarkdownString, MarkdownStringSendMessage};
/// use teloxide::{Bot, prelude::Requester, types::ChatId};
///
/// async fn send_markdown_example(bot: Bot, chat_id: ChatId) {
///     // Create a MarkdownString (safely escaped)
///     let message = MarkdownString::escape("Hello *world*!");
///     
///     // Use the trait method - automatically sets ParseMode::MarkdownV2
///     let request = bot.send_message(chat_id, message);
///     request.await.unwrap();
/// }
/// ```
///
/// The trait allows you to use `Bot::send_message` with `MarkdownString` parameters
/// while automatically applying the correct parse mode, making it safer and more
/// convenient than manually setting the parse mode each time.
#[allow(async_fn_in_trait)]
pub trait MarkdownStringMessage: Requester {
    /// Send a message with MarkdownString content
    ///
    /// This method has the same signature as teloxide's `Bot::send_message`,
    /// but accepts a MarkdownString instead of regular text and automatically
    /// sets the parse mode to MarkdownV2.
    async fn markdown_message<C>(
        &self,
        chat_id: C,
        message_id: Option<MessageId>,
        text: MarkdownString,
    ) -> ResponseResult<Message>
    where
        C: Into<Recipient>;

    fn send_markdown_message<C>(
        &self,
        chat_id: C,
        text: MarkdownString,
    ) -> teloxide::requests::JsonRequest<SendMessage>
    where
        C: Into<Recipient>;

    fn edit_markdown_message_text<C>(
        &self,
        chat_id: C,
        message_id: MessageId,
        text: MarkdownString,
    ) -> <Self as Requester>::EditMessageText
    where
        C: Into<Recipient>;
}

/// Implementation of MarkdownStringSendMessage for teloxide Bot
impl MarkdownStringMessage for Bot {
    fn send_markdown_message<C>(
        &self,
        chat_id: C,
        text: MarkdownString,
    ) -> teloxide::requests::JsonRequest<SendMessage>
    where
        C: Into<Recipient>,
    {
        self.send_message(chat_id, text)
            .parse_mode(ParseMode::MarkdownV2)
    }

    fn edit_markdown_message_text<C>(
        &self,
        chat_id: C,
        message_id: MessageId,
        text: MarkdownString,
    ) -> <Self as Requester>::EditMessageText
    where
        C: Into<Recipient>,
    {
        self.edit_message_text(chat_id, message_id, text)
            .parse_mode(MarkdownV2)
    }
    async fn markdown_message<C>(
        &self,
        chat_id: C,
        message_id: Option<MessageId>,
        text: MarkdownString,
    ) -> ResponseResult<Message>
    where
        C: Into<Recipient>,
    {
        if let Some(message_id) = message_id {
            self.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::MarkdownV2)
                .await
        } else {
            self.send_message(chat_id, text)
                .parse_mode(ParseMode::MarkdownV2)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{markdown_format, markdown_string};

    #[test]
    fn test_escape_constructor() {
        // Test basic escaping
        let markdown = MarkdownString::escape("Hello world");
        assert_eq!(markdown.as_str(), "Hello world");

        // Test escaping special characters
        let markdown = MarkdownString::escape("Hello! *bold* _italic_ `code`");
        assert_eq!(
            markdown.as_str(),
            "Hello\\! \\*bold\\* \\_italic\\_ \\`code\\`"
        );

        // Test escaping all reserved characters
        let markdown = MarkdownString::escape("!.-+=>#{}[]()~|");
        assert_eq!(
            markdown.as_str(),
            "\\!\\.\\-\\+\\=\\>\\#\\{\\}\\[\\]\\(\\)\\~\\|"
        );
    }

    #[test]
    fn test_new_constructor() {
        // Test creating an empty MarkdownString
        let markdown = MarkdownString::new();
        assert_eq!(markdown.as_str(), "");

        // Test that it's equivalent to escaping an empty string
        let escaped_empty = MarkdownString::escape("");
        assert_eq!(markdown.as_str(), escaped_empty.as_str());
    }

    #[test]
    fn test_default_constructor() {
        // Test creating an empty MarkdownString using Default
        let markdown = MarkdownString::default();
        assert_eq!(markdown.as_str(), "");

        // Test that it's equivalent to new()
        let new_markdown = MarkdownString::new();
        assert_eq!(markdown.as_str(), new_markdown.as_str());

        // Test using Default::default()
        let default_markdown: MarkdownString = Default::default();
        assert_eq!(default_markdown.as_str(), "");
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
        let markdown = markdown_string!("Hello *world*\\!");
        assert_eq!(markdown.as_str(), "Hello *world*\\!");
    }

    #[test]
    fn test_markdown_macro_with_arguments() {
        let name = "John";
        let markdown = markdown_format!("Hello *{}*\\!", name);
        assert_eq!(markdown.as_str(), "Hello *John*\\!");
    }

    #[test]
    fn test_markdown_macro_with_special_chars_in_args() {
        let text = "special! chars* here_";
        let markdown = markdown_format!("Message: `{}`", text);
        assert_eq!(markdown.as_str(), "Message: `special\\! chars\\* here\\_`");
    }

    #[test]
    fn test_markdown_macro_complex() {
        let user = "Alice";
        let amount = 100;
        let category = "food*";
        let markdown = markdown_format!(
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
    fn test_from_string_into_markdownstring() {
        let s = "Hello *world*".to_string();
        let markdown: MarkdownString = s.into();
        // Should be escaped since From implementations use escape()
        assert_eq!(markdown.as_str(), "Hello \\*world\\*");
    }

    #[test]
    fn test_from_str_into_markdownstring() {
        let s = "Hello *world*";
        let markdown: MarkdownString = s.into();
        // Should be escaped since From implementations use escape()
        assert_eq!(markdown.as_str(), "Hello \\*world\\*");
    }

    #[test]
    fn test_add_operation() {
        // Test adding two MarkdownStrings
        let part1 = MarkdownString::escape("Hello ");
        let part2 = MarkdownString::escape("world!");
        let combined = part1 + part2;
        assert_eq!(combined.as_str(), "Hello world\\!");

        // Test adding MarkdownString and &MarkdownString
        let part1 = MarkdownString::escape("Prefix ");
        let part2 = MarkdownString::test_template("*bold*");
        let combined = part1 + &part2;
        assert_eq!(combined.as_str(), "Prefix *bold*");

        // Test adding &MarkdownString and MarkdownString
        let part1 = MarkdownString::test_template("*Important:* ");
        let part2 = MarkdownString::escape("user input!");
        let combined = &part1 + part2;
        assert_eq!(combined.as_str(), "*Important:* user input\\!");

        // Test adding two &MarkdownString
        let part1 = MarkdownString::test_template("Start ");
        let part2 = MarkdownString::test_template("End");
        let combined = &part1 + &part2;
        assert_eq!(combined.as_str(), "Start End");

        // Test chaining multiple additions
        let part1 = MarkdownString::escape("User: ");
        let part2 = MarkdownString::escape("Alice");
        let part3 = MarkdownString::test_template(" said: *");
        let part4 = MarkdownString::escape("Hello!");
        let part5 = MarkdownString::test_template("*");
        let combined = part1 + part2 + part3 + part4 + part5;
        assert_eq!(combined.as_str(), "User: Alice said: *Hello\\!*");
    }

    #[test]
    fn test_direct_construction() {
        // Test direct construction without escaping (replaces raw test)
        let markdown = MarkdownString::test_template("Hello *world* with `code`!");
        assert_eq!(markdown.as_str(), "Hello *world* with `code`!");

        // Test with template placeholder
        let template = MarkdownString::test_template("Template with {} placeholder");
        assert_eq!(template.as_str(), "Template with {} placeholder");
    }

    #[test]
    fn test_usage_patterns_comparison() {
        // This test documents the different usage patterns and their effects

        // Pattern 1: Using escape() for safe user content (recommended for user input)
        let user_input = "User typed: Hello *world*!";
        let safe_content = MarkdownString::escape(user_input);
        assert_eq!(safe_content.as_str(), "User typed: Hello \\*world\\*\\!");

        // Pattern 2: Using direct construction for templates (when markdown_string! can't be used)
        let template = MarkdownString::test_template("Message: *{}* with balance ${}");
        let formatted = markdown_format!(template, "Alice", "50.00");
        assert_eq!(formatted.as_str(), "Message: *Alice* with balance $50\\.00");

        // Pattern 3: Using From/Into for safe content (auto-escapes)
        let string_content: MarkdownString = "Content with *markdown*".into();
        assert_eq!(string_content.as_str(), "Content with \\*markdown\\*");

        // Pattern 4: Direct construction for pre-validated content (test only)
        let direct = MarkdownString::test_template("Pre-validated *content*");
        assert_eq!(direct.as_str(), "Pre-validated *content*");

        // Show the difference between escape() and direct construction clearly
        let input = "Hello *bold* text!";
        let escaped = MarkdownString::escape(input);
        let direct = MarkdownString::test_template(input);
        assert_eq!(escaped.as_str(), "Hello \\*bold\\* text\\!");
        assert_eq!(direct.as_str(), "Hello *bold* text!");
    }

    #[test]
    fn test_markdown_format_with_markdownstring_args() {
        // Test that MarkdownString arguments work correctly with markdown_format!
        let template = MarkdownString::test_template("User: {} said: {}");

        // Create arguments as MarkdownString instances
        let username = MarkdownString::escape("Alice*"); // This should be escaped
        let pre_formatted = MarkdownString::test_template("*Important message*"); // This should not be escaped

        let result = markdown_format!(template, username, pre_formatted);

        // username should be escaped (Alice* -> Alice\*)
        // pre_formatted should remain as is (*Important message*)
        assert_eq!(result.as_str(), "User: Alice\\* said: *Important message*");
    }

    #[test]
    fn test_markdown_format_mixed_argument_types() {
        // Test mixing different argument types that all implement Into<MarkdownString>
        let template = MarkdownString::test_template("Mixed: {} and {} and {}");

        // Different argument types
        let string_arg = "Hello!"; // &str - will be escaped
        let markdown_arg = MarkdownString::escape("*bold*"); // MarkdownString - already escaped
        let number_arg = 42; // number - will be converted to string then escaped

        let result = markdown_format!(template, string_arg, markdown_arg, number_arg);

        // string_arg: "Hello!" -> "Hello\\!"
        // markdown_arg: already escaped "*bold*" -> "\\*bold\\*"
        // number_arg: 42 -> "42" (no special chars to escape)
        assert_eq!(result.as_str(), "Mixed: Hello\\! and \\*bold\\* and 42");
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
        let escaped_description =
            MarkdownString::escape(format!("User {} spent ${} on {}", user, amount, category));
        assert_eq!(
            escaped_description.as_str(),
            "User Alice spent $50\\.00 on groceries & food\\!"
        );

        // Using markdown macro for formatted messages (compile-time validated)
        let formatted_notification = markdown_format!(
            "💰 *Expense Added*\n\n*User:* {}\n*Amount:* ${}\n*Category:* `{}`\n*Date:* {}",
            user,
            amount,
            category,
            date
        );

        let expected = "💰 *Expense Added*\n\n*User:* Alice\n*Amount:* $50\\.00\n*Category:* `groceries & food\\!`\n*Date:* 2024\\-12\\-10";
        assert_eq!(formatted_notification.as_str(), expected);
    }

    #[test]
    fn test_markdown_format_macro_basic() {
        // Create a template using test helper (only for testing)
        let template = MarkdownString::test_template("Hello *{}*\\!");
        let name = "John";
        let result = markdown_format!(template, name);
        assert_eq!(result.as_str(), "Hello *John*\\!");
    }

    #[test]
    fn test_markdown_format_macro_with_special_chars() {
        let template = MarkdownString::test_template("Message: `{}`");
        let text = "special! chars* here_";
        let result = markdown_format!(template, text);
        assert_eq!(result.as_str(), "Message: `special\\! chars\\* here\\_`");
    }

    #[test]
    fn test_markdown_format_macro_complex() {
        let template =
            MarkdownString::test_template("*User:* {} \n*Amount:* ${} \n*Category:* `{}`");
        let user = "Alice";
        let amount = 100.50;
        let category = "food & drinks!";
        let result = markdown_format!(template, user, amount, category);
        assert_eq!(
            result.as_str(),
            "*User:* Alice \n*Amount:* $100\\.5 \n*Category:* `food & drinks\\!`"
        );
    }

    #[test]
    fn test_markdown_format_macro_no_args() {
        let template = MarkdownString::test_template("Simple message without placeholders\\.");
        let result = markdown_format!(template);
        assert_eq!(result.as_str(), "Simple message without placeholders\\.");
    }

    #[test]
    fn test_markdown_format_macro_multiple_same_placeholder() {
        let template = MarkdownString::test_template("Hello {}\\! Nice to meet you, {}\\.");
        let name = "Bob*";
        let result = markdown_format!(template, name, name);
        assert_eq!(
            result.as_str(),
            "Hello Bob\\*\\! Nice to meet you, Bob\\*\\."
        );
    }

    #[test]
    fn test_markdown_format_with_valid_markdown_template() {
        // This test shows using a properly formatted markdown template
        // Here we construct the template manually to have valid markdown
        let template = MarkdownString::test_template("Hello *{}*\\! Your balance is ${}");
        let name = "Alice";
        let balance = 42.50;
        let result = markdown_format!(template, name, balance);
        assert_eq!(result.as_str(), "Hello *Alice*\\! Your balance is $42\\.5");
    }

    #[test]
    fn test_markdown_format_vs_markdown_macro() {
        // Test comparing markdown_format! with direct formatting
        let user = "Alice";
        let amount = "50.00";

        // Using markdown_string! macro with direct args
        let direct_result = markdown_format!("*User:* {} spent ${}", user, amount);

        // Using markdown_format! macro with pre-constructed template
        let template = MarkdownString::test_template("*User:* {} spent ${}");
        let format_result = markdown_format!(template, user, amount);

        assert_eq!(direct_result.as_str(), format_result.as_str());
        assert_eq!(format_result.as_str(), "*User:* Alice spent $50\\.00");
    }

    #[test]
    fn test_markdown_format_with_escaped_template() {
        // Test using a template created with escape, but for content without placeholders
        let safe_content = "Hello! This is safe content: *bold* _italic_";
        let escaped_template = MarkdownString::escape(safe_content);
        let result = markdown_format!(escaped_template);
        assert_eq!(
            result.as_str(),
            "Hello\\! This is safe content: \\*bold\\* \\_italic\\_"
        );
    }

    #[test]
    fn test_markdown_format_example_usage() {
        // Example usage showing how the macro would be used in practice
        let template = MarkdownString("Hello *{}*\\! Your balance is: ${}\\.".to_string());

        // Use the template with arguments that contain special characters
        let name = "Alice & Bob";
        let balance = 123.45;

        let result = markdown_format!(template, name, balance);

        // Expected: Alice & Bob gets escaped to Alice & Bob (ampersand is escaped)
        // 123.45 gets escaped to 123\.45 (period is escaped)
        let expected = "Hello *Alice & Bob*\\! Your balance is: $123\\.45\\.";
        assert_eq!(result.as_str(), expected);
    }

    #[test]
    fn test_markdown_format_with_into_trait() {
        // Test that the macro works with MarkdownString templates

        // Using MarkdownString directly (constructed without escaping)
        let markdown_template = MarkdownString::test_template("Using MarkdownString: {}");
        let result1 = markdown_format!(markdown_template, "test");
        assert_eq!(result1.as_str(), "Using MarkdownString: test");

        // Using direct construction for templates (use with caution - no compile-time validation)
        let direct_template = MarkdownString::test_template("Using direct template: {}");
        let result2 = markdown_format!(direct_template, "test");
        assert_eq!(result2.as_str(), "Using direct template: test");

        // Using escaped content as template (placeholders get escaped)
        let escaped_template = MarkdownString::escape("This {} won't work as template");
        let result3 = markdown_format!(escaped_template, "arg");
        // The {} gets escaped to \{\}, so "arg" doesn't replace anything
        assert_eq!(result3.as_str(), "This \\{\\} won't work as template");

        // Show the correct way to use templates with markdown formatting
        let proper_template = MarkdownString::test_template("Proper *{}* template with `{}`");
        let result4 = markdown_format!(proper_template, "bold", "code");
        assert_eq!(result4.as_str(), "Proper *bold* template with `code`");
    }

    // The following tests verify that the markdown_string! macro would catch invalid syntax
    // at compile time. These are included as documentation but commented out since
    // they would actually fail compilation.

    /*
    #[test]
    fn test_compile_time_validation_unmatched_bold() {
        // This would fail at compile time:
        // let markdown = markdown_string!("*unmatched bold");
    }

    #[test]
    fn test_compile_time_validation_unescaped_exclamation() {
        // This would fail at compile time:
        // let markdown = markdown_string!("Hello!");
    }

    #[test]
    fn test_compile_time_validation_unmatched_italic() {
        // This would fail at compile time:
        // let markdown = markdown_string!("_unmatched italic");
    }

    #[test]
    fn test_compile_time_validation_unmatched_code() {
        // This would fail at compile time:
        // let markdown = markdown_string!("`unmatched code");
    }
    */

    // Note: We can't easily test the MarkdownStringSendMessage trait without
    // setting up a real Bot instance, but we can test that the types are correct
    #[test]
    fn test_markdown_string_send_message_trait_exists() {
        // This test ensures the trait is properly defined and accessible
        use crate::api::markdown::string::MarkdownStringMessage;

        // If this compiles, the trait is properly defined
        fn _test_trait_bound<T: MarkdownStringMessage>(_bot: T) {}

        // Test that MarkdownString can be created for the trait method
        let _message = MarkdownString::escape("Test message");
    }
}
