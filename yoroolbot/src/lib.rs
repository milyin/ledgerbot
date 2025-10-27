//! Yoroolbot - A library crate for yoroolbot functionality

// Private API modules
pub(crate) mod api;

// Public markdown module with re-exports
pub mod markdown {
    // Re-export types and traits from internal API
    pub use crate::api::markdown::{
        string::{MarkdownString, MarkdownStringMessage},
        validate::validate_markdownv2_format,
    };
}

// Public command_trait module with re-exports
pub mod command_trait {
    // Re-export types and traits from internal API
    pub use crate::api::command_trait::{
        CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand, ParseCommandArg,
    };
}

// Public storage module with re-exports
pub mod storage {
    // Re-export types and traits from internal API
    pub use crate::api::storage::callback_data_storage::{
        CallbackDataStorage, CallbackDataStorageTrait,
    };
}
