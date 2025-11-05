use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Error type for Category parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCategoryError(String);

impl fmt::Display for ParseCategoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseCategoryError {}

/// Represents a category for expense classification
/// Categories can be user-defined names or special categories:
/// - None: Used in reports to show summary with category buttons
/// - Other: Uncategorized expenses
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Category {
    /// User-defined category with a name
    Category(String),
    /// Special category for showing summary with category selection
    None,
    /// Special category for uncategorized expenses
    #[default]
    Other,
}

impl Category {
    /// Parse category from string
    /// Only allows single word without spaces and special syntax characters
    /// Special strings: "<none>" maps to Category::None, "<other>" maps to Category::Other
    pub fn from_string(s: &str) -> Result<Self, String> {
        // Special case for <none>
        if s == "<none>" {
            return Ok(Category::None);
        }

        // Special case for <other>
        if s == "<other>" {
            return Ok(Category::Other);
        }

        // Validate: must be a single word without spaces or special characters
        if s.is_empty() {
            return Err("Category name cannot be empty".to_string());
        }

        if s.contains(char::is_whitespace) {
            return Err(format!(
                "Category name '{}' cannot contain spaces. Use a single word.",
                s
            ));
        }

        // Check for special syntax characters that might interfere with markdown or telegram
        let forbidden_chars = [
            '*', '_', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.',
            '!', '\\',
        ];
        for ch in forbidden_chars {
            if s.contains(ch) {
                return Err(format!(
                    "Category name '{}' cannot contain special character '{}'",
                    s, ch
                ));
            }
        }

        Ok(Category::Category(s.to_string()))
    }

    /// Check if this is the None category
    pub fn is_none(&self) -> bool {
        matches!(self, Category::None)
    }

    /// Check if this is the Other category
    pub fn is_other(&self) -> bool {
        matches!(self, Category::Other)
    }

    /// Get the category name as a string slice
    /// Returns "<none>" for None variant, "<other>" for Other variant
    pub fn as_str(&self) -> &str {
        match self {
            Category::Category(name) => name.as_str(),
            Category::None => "<none>",
            Category::Other => "<other>",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Category {
    type Err = ParseCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s).map_err(ParseCategoryError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_valid() {
        let cat = Category::from_string("Food").unwrap();
        assert_eq!(cat, Category::Category("Food".to_string()));

        let cat2 = Category::from_string("Transport").unwrap();
        assert_eq!(cat2, Category::Category("Transport".to_string()));
    }

    #[test]
    fn test_from_string_none() {
        let cat = Category::from_string("<none>").unwrap();
        assert_eq!(cat, Category::None);
    }

    #[test]
    fn test_from_string_other() {
        let cat = Category::from_string("<other>").unwrap();
        assert_eq!(cat, Category::Other);
    }

    #[test]
    fn test_from_string_invalid_spaces() {
        assert!(Category::from_string("Food and Drinks").is_err());
        assert!(Category::from_string("My Category").is_err());
    }

    #[test]
    fn test_from_string_invalid_special_chars() {
        assert!(Category::from_string("Food*").is_err());
        assert!(Category::from_string("Food_").is_err());
        assert!(Category::from_string("Food[1]").is_err());
        assert!(Category::from_string("Food.2").is_err());
    }

    #[test]
    fn test_from_string_empty() {
        assert!(Category::from_string("").is_err());
    }

    #[test]
    fn test_display() {
        let cat = Category::Category("Food".to_string());
        assert_eq!(format!("{}", cat), "Food");

        let none = Category::None;
        assert_eq!(format!("{}", none), "<none>");

        let other = Category::Other;
        assert_eq!(format!("{}", other), "<other>");
    }

    #[test]
    fn test_as_str() {
        let cat = Category::Category("Food".to_string());
        assert_eq!(cat.as_str(), "Food");

        let none = Category::None;
        assert_eq!(none.as_str(), "<none>");

        let other = Category::Other;
        assert_eq!(other.as_str(), "<other>");
    }

    #[test]
    fn test_is_none() {
        let cat = Category::Category("Food".to_string());
        assert!(!cat.is_none());

        let none = Category::None;
        assert!(none.is_none());

        let other = Category::Other;
        assert!(!other.is_none());
    }

    #[test]
    fn test_is_other() {
        let cat = Category::Category("Food".to_string());
        assert!(!cat.is_other());

        let none = Category::None;
        assert!(!none.is_other());

        let other = Category::Other;
        assert!(other.is_other());
    }

    #[test]
    fn test_fromstr_trait() {
        let cat: Category = "Food".parse().unwrap();
        assert_eq!(cat, Category::Category("Food".to_string()));

        let none: Category = "<none>".parse().unwrap();
        assert_eq!(none, Category::None);

        let other: Category = "<other>".parse().unwrap();
        assert_eq!(other, Category::Other);
    }

    #[test]
    fn test_default() {
        let cat = Category::default();
        assert_eq!(cat, Category::Other);
    }

    #[test]
    fn test_ordering() {
        let cat1 = Category::Category("A".to_string());
        let cat2 = Category::Category("B".to_string());
        let none = Category::None;
        let other = Category::Other;

        assert!(cat1 < cat2);
        assert!(cat1 < none); // Category variant comes first in enum order
        assert!(none < other); // None comes before Other in enum order
        assert!(other > cat1);
    }
}
