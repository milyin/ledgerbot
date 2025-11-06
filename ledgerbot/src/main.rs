mod batch;
mod commands;
mod config;
mod handlers;
pub mod menus;
mod storages;
mod utils;

use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use config::Args;
use handlers::{handle_callback_query, handle_text_message};
use teloxide::prelude::*;
use yoroolbot::storage::FilesystemYamlStore;

use crate::storages::{
    CategoryData, CategoryStorage, ExpenseData, ExpenseStorage, ShareData, Stores,
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
        let share_dir = base_dir.join("shares");

        let category_store = FilesystemYamlStore::<CategoryData>::new(categories_dir);
        let expense_store = FilesystemYamlStore::<ExpenseData>::new(expenses_dir);
        let share_store = FilesystemYamlStore::<ShareData>::new(share_dir);

        Stores::new()
            .categories_store(category_store)
            .expenses_store(expense_store)
            .shares_store(share_store)
    } else {
        // Use in-memory storage
        log::info!("Using in-memory storage");
        Stores::new()
    };

    // Wrap storage in Arc for use throughout the bot
    let storage: Arc<Stores> = Arc::new(storage);

    // Create handler using modern teloxide patterns
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                // Route all text messages (including commands) to handle_text_message
                // which can parse and execute multiple commands from a single message
                .branch(
                    dptree::filter(|msg: Message| msg.text().is_some())
                        .endpoint(handle_text_message),
                ),
        )
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
