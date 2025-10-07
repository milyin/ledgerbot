use teloxide::{
    prelude::*,
    types::Message,
    utils::command::BotCommands,
};

use super::Command;

/// Display help message with inline keyboard buttons
pub async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = format!(
        "To add expenses forward messages or send text with lines in format:\n\
        <code>[&lt;yyyy-mm-dd&gt;] &lt;description&gt; &lt;amount&gt;</code>\n\n\
        {commands}",
        commands = Command::descriptions()
    );

    // Send message with both inline keyboard (for buttons in message) and reply keyboard (menu button)
    bot.send_message(msg.chat.id, help_text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}
