//! Yoroolbot - A library crate for yoroolbot functionality

// Private API modules
pub(crate)mod api;

// Public markdown module with re-exports
pub mod markdown {
    // Re-export types and traits from internal API
    pub use crate::api::markdown::{
        string::{MarkdownString, MarkdownStringSendMessage},
        validate::validate_markdownv2_format,
    };
}
