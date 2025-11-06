use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Error type for ShareUsername parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseShareUsernameError(String);

impl fmt::Display for ParseShareUsernameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseShareUsernameError {}

/// Represents a validated Telegram username for sharing
/// Usernames must:
/// - Start with '@'
/// - Be 5-32 characters long (after the '@')
/// - Contain only letters (A-Z, case-insensitive), digits (0-9), and underscores
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShareUsername(String);

impl Default for ShareUsername {
    fn default() -> Self {
        // Return a valid but placeholder username
        ShareUsername("@_____".to_string())
    }
}

impl ShareUsername {
    /// Parse and validate a Telegram username
    /// Username must start with '@' and follow Telegram's username rules
    pub fn from_string(s: &str) -> Result<Self, String> {
        // Must start with '@'
        if !s.starts_with('@') {
            return Err(format!(
                "Username '{}' must start with '@'. Example: @username",
                s
            ));
        }

        // Extract username without '@'
        let username_part = &s[1..];

        // Check length (5-32 characters after '@')
        let len = username_part.len();
        if len < 5 {
            return Err(format!(
                "Username '{}' is too short. Must be at least 5 characters after '@'.",
                s
            ));
        }
        if len > 32 {
            return Err(format!(
                "Username '{}' is too long. Must be at most 32 characters after '@'.",
                s
            ));
        }

        // Check that all characters are valid (letters, digits, underscores)
        for ch in username_part.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(format!(
                    "Username '{}' contains invalid character '{}'. Only letters, digits, and underscores are allowed.",
                    s, ch
                ));
            }
        }

        Ok(ShareUsername(s.to_string()))
    }

    /// Get the username as a string slice (includes the '@' prefix)
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ShareUsername {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ShareUsername {
    type Err = ParseShareUsernameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s).map_err(ParseShareUsernameError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let username = ShareUsername::from_string("@user123").unwrap();
        assert_eq!(username.as_str(), "@user123");

        let username2 = ShareUsername::from_string("@Alice_Bot").unwrap();
        assert_eq!(username2.as_str(), "@Alice_Bot");
    }

    #[test]
    fn test_missing_at_sign() {
        let result = ShareUsername::from_string("user123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must start with '@'"));
    }

    #[test]
    fn test_too_short() {
        let result = ShareUsername::from_string("@abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    #[test]
    fn test_too_long() {
        let result = ShareUsername::from_string("@abcdefghijklmnopqrstuvwxyz1234567");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too long"));
    }

    #[test]
    fn test_invalid_characters() {
        assert!(ShareUsername::from_string("@user-name").is_err());
        assert!(ShareUsername::from_string("@user.name").is_err());
        assert!(ShareUsername::from_string("@user@name").is_err());
        assert!(ShareUsername::from_string("@user name").is_err());
    }

    #[test]
    fn test_valid_edge_cases() {
        // Exactly 5 characters after '@'
        assert!(ShareUsername::from_string("@abcde").is_ok());

        // Exactly 32 characters after '@'
        assert!(ShareUsername::from_string("@abcdefghijklmnopqrstuvwxyz123456").is_ok());

        // Mix of letters, digits, underscores
        assert!(ShareUsername::from_string("@Test_User_123").is_ok());
    }

    #[test]
    fn test_fromstr_trait() {
        let username: ShareUsername = "@user123".parse().unwrap();
        assert_eq!(username.as_str(), "@user123");

        let result: Result<ShareUsername, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let username = ShareUsername::from_string("@user123").unwrap();
        assert_eq!(format!("{}", username), "@user123");
    }

    #[test]
    fn test_ordering() {
        let user1 = ShareUsername::from_string("@alice").unwrap();
        let user2 = ShareUsername::from_string("@bobby").unwrap();
        assert!(user1 < user2);
    }
}
