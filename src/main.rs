use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::{sync::Mutex, time::sleep};

const PREDEFINED_BOT_TOKEN: Option<&str> = option_env!("PREDEFINED_BOT_TOKEN");
const BOT_TOKEN_HELP: &str = if PREDEFINED_BOT_TOKEN.is_some() {
    "Environment variable name containing the bot token. If not set, uses precompiled token"
} else {
    "Environment variable name containing the bot token (required)"
};

/// A Telegram bot that calculates expenses from forwarded messages
#[derive(Parser, Debug)]
#[command(name = "ledgerbot")]
#[command(about = "A Telegram bot that calculates expenses", long_about = None)]
struct Args {
    #[arg(long, help = BOT_TOKEN_HELP)]
    bot_token_env: Option<String>,
}

/// Bot commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this help")]
    Help,
    #[command(description = "start the bot")]
    Start,
    #[command(description = "show all expenses")]
    List,
    #[command(description = "clear all expenses")]
    Clear,
}

// Per-chat storage for expenses - each chat has its own expense HashMap
type ExpenseStorage = Arc<Mutex<HashMap<ChatId, HashMap<String, f64>>>>;

// Batch processing state
#[derive(Clone)]
struct BatchState {
    messages_count: usize,
    records_count: usize,
    total_sum: f64,
}

// Per-chat batch storage - each chat has its own batch state
type BatchStorage = Arc<Mutex<HashMap<ChatId, BatchState>>>;

const BATCH_TIMEOUT_SECONDS: u64 = 1; // Report after N seconds of inactivity

// Function to parse expense lines from a message
fn parse_expenses(text: &str) -> Vec<(String, f64)> {
    let mut expenses = Vec::new();
    
    // Regex pattern to match "<any text> <number>"
    // This captures text followed by a space and then a number (integer or decimal)
    let re = Regex::new(r"^(.+?)\s+(\d+(?:\.\d+)?)$").unwrap();
    
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        if let Some(captures) = re.captures(line) {
            let description = captures[1].trim().to_string();
            if let Ok(amount) = captures[2].parse::<f64>() {
                expenses.push((description, amount));
            }
        }
    }
    
    expenses
}

// Function to format expenses as a readable list
fn format_expenses_list(expenses: &HashMap<String, f64>) -> String {
    if expenses.is_empty() {
        return "No expenses recorded yet.".to_string();
    }
    
    let mut result = "📊 **Current Expenses:**\n\n".to_string();
    let mut total = 0.0;
    
    for (description, amount) in expenses.iter() {
        result.push_str(&format!("• {} - {:.2}\n", description, amount));
        total += amount;
    }
    
    result.push_str(&format!("\n💰 **Total: {:.2}**", total));
    result
}

// Command handlers
async fn help_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = "💡 **Expense Bot Help**\n\n\
        **How to add expenses:**\n\
        Forward messages or send text with lines in format:\n\
        `<description> <amount>`\n\n\
        **Examples:**\n\
        `Coffee 5.50`\n\
        `Lunch 12.00`\n\
        `Bus ticket 2.75`\n\n\
        **Commands:**\n\
        `/list` - Show all expenses\n\
        `/clear` - Clear all expenses\n\
        `/help` - Show this help\n\n\
        **Note:** The bot will collect your expense messages and report a summary after a few seconds of inactivity.";
    
    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}

async fn list_command(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let expenses_list = {
        let storage_guard = storage.lock().await;
        let chat_expenses = storage_guard.get(&chat_id)
            .cloned()
            .unwrap_or_default();
        format_expenses_list(&chat_expenses)
    };
    
    bot.send_message(chat_id, expenses_list).await?;
    Ok(())
}

async fn clear_command(
    bot: Bot,
    msg: Message,
    storage: ExpenseStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    {
        let mut storage_guard = storage.lock().await;
        storage_guard.remove(&chat_id);
    }
    
    bot.send_message(chat_id, "🗑️ All expenses cleared!").await?;
    Ok(())
}

// Unified command handler
async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    storage: ExpenseStorage,
) -> ResponseResult<()> {
    match cmd {
        Command::Help | Command::Start => help_command(bot, msg).await,
        Command::List => list_command(bot, msg, storage).await,
        Command::Clear => clear_command(bot, msg, storage).await,
    }
}

async fn handle_text_message(
    bot: Bot, 
    msg: Message, 
    storage: ExpenseStorage,
    batch_storage: BatchStorage,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    
    if let Some(text) = msg.text() {
        // Parse expenses from the message
        let parsed_expenses = parse_expenses(text);
        
        if !parsed_expenses.is_empty() {
            // Store the expenses in chat-specific storage
            {
                let mut storage_guard = storage.lock().await;
                let chat_expenses = storage_guard.entry(chat_id).or_default();
                for (description, amount) in parsed_expenses.iter() {
                    chat_expenses.insert(description.clone(), *amount);
                }
            }

            // Update batch state for this chat
            let total_parsed: f64 = parsed_expenses.iter().map(|(_, amount)| amount).sum();
            let is_first_message;
            
            {
                let mut batch_guard = batch_storage.lock().await;
                match batch_guard.get_mut(&chat_id) {
                    Some(state) => {
                        // Update existing batch for this chat
                        state.messages_count += 1;
                        state.records_count += parsed_expenses.len();
                        state.total_sum += total_parsed;
                        is_first_message = false;
                    }
                    None => {
                        // Start new batch for this chat
                        batch_guard.insert(chat_id, BatchState {
                            messages_count: 1,
                            records_count: parsed_expenses.len(),
                            total_sum: total_parsed,
                        });
                        is_first_message = true;
                    }
                }
            }
            
            // Start timeout task only for the first message in batch
            if is_first_message {
                let batch_clone = batch_storage.clone();
                let bot_clone = bot.clone();
                tokio::spawn(async move {
                    sleep(tokio::time::Duration::from_secs(BATCH_TIMEOUT_SECONDS)).await;
                    send_batch_report(bot_clone, batch_clone, chat_id).await;
                });
            }
        }
    }
    
    Ok(())
}

async fn send_batch_report(bot: Bot, batch_storage: BatchStorage, target_chat_id: ChatId) {
    let batch_data = {
        let mut batch_guard = batch_storage.lock().await;
        // Simply remove and return the batch state if it exists
        batch_guard.remove(&target_chat_id)
    };

    if let Some(state) = batch_data {
        let report = format!(
            "📊 **Batch Summary Report**\n\n\
            📨 Messages processed: {}\n\
            📝 Records parsed: {}\n\
            💰 Total amount: {:.2}\n\n\
            Use `/list` to see all expenses.",
            state.messages_count,
            state.records_count,
            state.total_sum
        );
        
        if let Err(e) = bot.send_message(target_chat_id, report).await {
            log::error!("Failed to send batch report: {}", e);
        }
    }
}



#[tokio::main]
async fn main() {
    let args = Args::parse();

    pretty_env_logger::init();
    log::info!("Starting expense calculation bot...");

    let token = if let Some(env_name) = args.bot_token_env {
        std::env::var(&env_name)
            .unwrap_or_else(|_| panic!("Environment variable {} not found", env_name))
    } else if let Some(predefined) = PREDEFINED_BOT_TOKEN {
        predefined.to_string()
    } else {
        panic!("No bot token provided and no precompiled token available. Use --bot-token-env")
    };

    let bot = Bot::new(token);
    
    // Initialize shared expense storage
    let storage: ExpenseStorage = Arc::new(Mutex::new(HashMap::new()));
    
    // Initialize batch storage
    let batch_storage: BatchStorage = Arc::new(Mutex::new(HashMap::new()));

    // Create handler using modern teloxide patterns
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer)
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some())
                .endpoint(handle_text_message)
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, batch_storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
