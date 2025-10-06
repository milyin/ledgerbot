mod batch;
mod commands;
mod config;
mod handlers;
mod parser;
mod storage;

use clap::Parser;
use teloxide::prelude::*;

use batch::create_batch_storage;
use commands::{answer, Command};
use config::Args;
use handlers::{handle_callback_query, handle_text_message};
use storage::{create_category_storage, create_storage};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    pretty_env_logger::init();
    log::info!("Starting expense calculation bot...");

    let token = args.get_token();
    let bot = Bot::new(token);

    // Initialize shared expense storage
    let storage = create_storage();

    // Initialize category storage
    let category_storage = create_category_storage();

    // Initialize batch storage
    let batch_storage = create_batch_storage();

    // Create handler using modern teloxide patterns
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(dptree::entry().filter_command::<Command>().endpoint(answer))
                .branch(dptree::filter(|msg: Message| msg.text().is_some()).endpoint(handle_text_message))
        )
        .branch(Update::filter_callback_query().endpoint(handle_callback_query));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, category_storage, batch_storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
