mod batch;
mod commands;
mod config;
mod handlers;
mod macros;
mod markdown_string;
mod parser;
mod storage;
mod storage_traits;

use clap::Parser;
use std::sync::Arc;
use std::path::PathBuf;
use teloxide::prelude::*;

use config::Args;
use handlers::{handle_callback_query, handle_text_message};
use storage::{Storage, PersistentCategoryStorage};
use storage_traits::StorageTrait;

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
        let storage_dir = storage_path.unwrap_or_else(|| PathBuf::from("categories"));
        log::info!("Using persistent category storage in directory: {:?}", storage_dir);
        Storage::new().categories_storage(PersistentCategoryStorage::new(storage_dir))
    } else {
        // Use in-memory storage
        log::info!("Using in-memory category storage");
        Storage::new()
    };

    // Wrap storage in Arc<dyn StorageTrait> for use throughout the bot
    let storage_trait: Arc<dyn StorageTrait> = Arc::new(storage);

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
        .dependencies(dptree::deps![storage_trait])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
