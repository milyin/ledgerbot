use clap::Parser;

pub const PREDEFINED_BOT_TOKEN_RELEASE: Option<&str> = option_env!("PREDEFINED_BOT_TOKEN_RELEASE");
pub const PREDEFINED_BOT_TOKEN_DEBUG: Option<&str> = option_env!("PREDEFINED_BOT_TOKEN_DEBUG");
pub const PREDEFINED_BOT_TOKEN: Option<&str> = if cfg!(debug_assertions) {
    PREDEFINED_BOT_TOKEN_DEBUG
} else {
    PREDEFINED_BOT_TOKEN_RELEASE
};
pub const BOT_TOKEN_HELP: &str = if PREDEFINED_BOT_TOKEN_RELEASE.is_some() {
    "Environment variable name containing the bot token. If not set, uses precompiled token"
} else {
    "Environment variable name containing the bot token (required)"
};

pub const BATCH_TIMEOUT_SECONDS: u64 = 1; // Report after N seconds of inactivity

/// A Telegram bot that calculates expenses from forwarded messages
#[derive(Parser, Debug)]
#[command(name = "ledgerbot")]
#[command(about = "A Telegram bot that calculates expenses", long_about = None)]
pub struct Args {
    #[arg(long, help = BOT_TOKEN_HELP)]
    pub bot_token_env: Option<String>,
}

impl Args {
    /// Get the bot token from CLI args or predefined token
    pub fn get_token(&self) -> String {
        if let Some(env_name) = &self.bot_token_env {
            std::env::var(env_name)
                .unwrap_or_else(|_| panic!("Environment variable {} not found", env_name))
        } else if let Some(predefined) = PREDEFINED_BOT_TOKEN {
            predefined.to_string()
        } else {
            panic!("No bot token provided and no precompiled token available. Use --bot-token-env")
        }
    }
}
