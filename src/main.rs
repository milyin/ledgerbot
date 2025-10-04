use teloxide::{prelude::Requester as _, types::Message, Bot};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    // let bot = Bot::from_env();
    const DEFAULT_TOKEN: &str = "your_default_bot_token_here";
    
    let token = option_env!("TELOXIDE_TOKEN").unwrap_or(DEFAULT_TOKEN);
    let bot = Bot::new(token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}