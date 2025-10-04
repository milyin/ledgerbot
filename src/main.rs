use teloxide::{prelude::Requester as _, types::Message, Bot};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    // let bot = Bot::from_env();
    let bot = Bot::new(env!("TELOXIDE_TOKEN"));

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}