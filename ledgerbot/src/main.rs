mod batch;
mod commands;
mod config;
mod data_store;
mod handlers;
pub mod menus;
mod storages;
mod ui;
mod utils;

use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use config::Args;
use handlers::{
    filter_command_prefixed, handle_callback_query, handle_command_message, handle_text_message,
    is_direct_command_message,
};
use telluride::{command::CallbackKey, data_store::InMemStore};
use teloxide::prelude::*;

use crate::data_store::FilesystemYamlStore;
use crate::{
    commands::Command,
    storages::{CategoryData, ExpenseData, FollowersData, Stores},
};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    pretty_env_logger::init();
    log::info!("Starting expense calculation bot...");

    let token = args.get_token();
    let bot = Bot::new(token);

    // Initialize main storage based on CLI arguments
    let storage = if let Some(storage_path) = args.persistent_storage {
        // Use persistent storage with provided path or default
        let base_dir = storage_path.unwrap_or_else(|| PathBuf::from("storage"));
        log::info!("Using persistent storage in directory: {:?}", base_dir);

        // Create subdirectories for different data types
        let categories_dir = base_dir.join("categories");
        let expenses_dir = base_dir.join("expenses");
        let followers_dir = base_dir.join("followers");

        let category_store = FilesystemYamlStore::<CategoryData>::new(categories_dir);
        let expense_store = FilesystemYamlStore::<ExpenseData>::new(expenses_dir);
        let followers_store = FilesystemYamlStore::<FollowersData>::new(followers_dir);

        Stores::new()
            .categories_store(category_store)
            .expenses_store(expense_store)
            .followers_store(followers_store)
    } else {
        // Use in-memory storage
        log::info!("Using in-memory storage");
        Stores::new()
    };

    // Wrap storage in Arc for use throughout the bot
    let storage: Arc<Stores> = Arc::new(storage);
    let callback_storage = Arc::new(InMemStore::<CallbackKey, Command>::new());

    // Route direct single-line commands through teloxide's command parser while
    // preserving ledgerbot's batch parser for multiline and forwarded input.
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter(is_direct_command_message)
                .chain(filter_command_prefixed::<commands::Command, _>())
                .endpoint(handle_command_message),
        )
        .branch(Update::filter_message().branch(
            dptree::filter(|msg: Message| msg.text().is_some()).endpoint(handle_text_message),
        ))
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, callback_storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
