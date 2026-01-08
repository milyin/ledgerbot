//! Yoroolbot - A library crate for yoroolbot functionality

// Private API modules
pub(crate) mod api;

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
    pub use crate::api::storage::{
        callback_data_storage::{
            ButtonData, CallbackData, CallbackDataStorage, CallbackDataStorageTrait,
            pack_callback_data, unpack_callback_data,
        },
        datastore::{DataStoreTrait, FilesystemYamlStore, InMemStore},
        utils::{decode_filename_to_key, encode_key_to_filename},
    };
}
