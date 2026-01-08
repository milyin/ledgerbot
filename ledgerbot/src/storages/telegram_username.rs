use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use telluride::{markdown::MarkdownString, markdown_format};

/// Error type for TelegramUsername parsing
#[derive(Debug, Clone)]
pub struct ParseTelegramUsernameError(MarkdownString);

impl fmt::Display for ParseTelegramUsernameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseTelegramUsernameError {}

/// Represents a validated Telegram username for sharing
/// Usernames must:
/// - Start with '@'
/// - Be 5-32 characters long (after the '@')
/// - Contain only letters (A-Z, case-insensitive), digits (0-9), and underscores
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TelegramUsername(String);

impl Default for TelegramUsername {
    fn default() -> Self {
        // Return a valid but placeholder username
        TelegramUsername("@_____".to_string())
    }
}

impl TelegramUsername {
    /// Parse and validate a Telegram username
    /// Username must start with '@' and follow Telegram's username rules
    pub fn from_string(s: &str) -> Result<Self, MarkdownString> {
        // Must start with '@'
        if !s.starts_with('@') {
            return Err(markdown_format!(
                "Username `{}` must start with `@`\\. Example: @username",
                s
            ));
        }

        // Extract username without '@'
        let username_part = &s[1..];

        // Check length (5-32 characters after '@')
        let len = username_part.len();
        if len < 5 {
            return Err(markdown_format!(
                "Username `{}` is too short\\. Must be at least 5 characters after `@`\\.",
                s
            ));
        }
        if len > 32 {
            return Err(markdown_format!(
                "Username `{}` is too long\\. Must be at most 32 characters after `@`\\.",
                s
            ));
        }

        // Check that all characters are valid (letters, digits, underscores)
        for ch in username_part.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(markdown_format!(
                    "Username `{}` contains invalid character `{}`\\. Only letters, digits, and underscores are allowed\\.",
                    s,
                    ch.to_string()
                ));
            }
        }

        Ok(TelegramUsername(s.to_string()))
    }

    /// Get the username as a string slice (includes the '@' prefix)
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for TelegramUsername {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TelegramUsername {
    type Err = ParseTelegramUsernameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s).map_err(ParseTelegramUsernameError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let username = TelegramUsername::from_string("@user123").unwrap();
        assert_eq!(username.as_str(), "@user123");

        let username2 = TelegramUsername::from_string("@Alice_Bot").unwrap();
        assert_eq!(username2.as_str(), "@Alice_Bot");
    }

    #[test]
    fn test_missing_at_sign() {
        let result = TelegramUsername::from_string("user123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must start"));
    }

    #[test]
    fn test_too_short() {
        let result = TelegramUsername::from_string("@abc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too short"));
    }

    #[test]
    fn test_too_long() {
        let result = TelegramUsername::from_string("@abcdefghijklmnopqrstuvwxyz1234567");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too long"));
    }

    #[test]
    fn test_invalid_characters() {
        assert!(TelegramUsername::from_string("@user-name").is_err());
        assert!(TelegramUsername::from_string("@user.name").is_err());
        assert!(TelegramUsername::from_string("@user@name").is_err());
        assert!(TelegramUsername::from_string("@user name").is_err());
    }

    #[test]
    fn test_valid_edge_cases() {
        // Exactly 5 characters after '@'
        assert!(TelegramUsername::from_string("@abcde").is_ok());

        // Exactly 32 characters after '@'
        assert!(TelegramUsername::from_string("@abcdefghijklmnopqrstuvwxyz123456").is_ok());

        // Mix of letters, digits, underscores
        assert!(TelegramUsername::from_string("@Test_User_123").is_ok());
    }

    #[test]
    fn test_fromstr_trait() {
        let username: TelegramUsername = "@user123".parse().unwrap();
        assert_eq!(username.as_str(), "@user123");

        let result: Result<TelegramUsername, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let username = TelegramUsername::from_string("@user123").unwrap();
        assert_eq!(format!("{}", username), "@user123");
    }

    #[test]
    fn test_ordering() {
        let user1 = TelegramUsername::from_string("@alice").unwrap();
        let user2 = TelegramUsername::from_string("@bobby").unwrap();
        assert!(user1 < user2);
    }
}
