mod batch;
mod commands;
mod config;
mod handlers;
mod parser;
mod storage;

use clap::Parser;
use teloxide::prelude::*;

use batch::create_batch_storage;
use config::Args;
use handlers::{handle_callback_query, handle_text_message};
use storage::Storage;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    pretty_env_logger::init();
    log::info!("Starting expense calculation bot...");

    let token = args.get_token();
    let bot = Bot::new(token);

    // Initialize main storage (holds expenses, categories, and filter state)
    let storage = Storage::new();

    // Initialize batch storage (separate from main storage for now)
    let batch_storage = create_batch_storage();

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
        .dependencies(dptree::deps![
            storage.clone(),
            batch_storage
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
