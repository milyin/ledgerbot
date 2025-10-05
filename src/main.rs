use clap::Parser;
use teloxide::{Bot, prelude::Requester as _, types::Message};

const PREDEFINED_BOT_TOKEN: Option<&str> = option_env!("PREDEFINED_BOT_TOKEN");
const BOT_TOKEN_HELP: &str = if PREDEFINED_BOT_TOKEN.is_some() {
    "Environment variable name containing the bot token. If not set, uses precompiled token"
} else {
    "Environment variable name containing the bot token (required)"
};

/// A simple Telegram bot that sends dice
#[derive(Parser, Debug)]
#[command(name = "ledgerbot")]
#[command(about = "A Telegram bot that sends dice", long_about = None)]
struct Args {
    #[arg(long, help = BOT_TOKEN_HELP)]
    bot_token_env: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let token = if let Some(env_name) = args.bot_token_env {
        std::env::var(&env_name)
            .unwrap_or_else(|_| panic!("Environment variable {} not found", env_name))
    } else if let Some(predefined) = PREDEFINED_BOT_TOKEN {
        predefined.to_string()
    } else {
        panic!("No bot token provided and no precompiled token available. Use --bot-token-env")
    };

    let bot = Bot::new(token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}
