use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use teloxide::{Bot, prelude::Requester as _, types::Message, RequestError};

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

// Shared state for storing expenses
type ExpenseStorage = Arc<Mutex<HashMap<String, f64>>>;

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

async fn handle_message(
    bot: Bot, 
    msg: Message, 
    storage: ExpenseStorage
) -> Result<(), RequestError> {
    if let Some(text) = msg.text() {
        // Check if this is a forwarded message or contains expense data
        let parsed_expenses = parse_expenses(text);
        
        if !parsed_expenses.is_empty() {
            // Store the expenses
            {
                let mut expenses = storage.lock().unwrap();
                for (description, amount) in parsed_expenses.iter() {
                    expenses.insert(description.clone(), *amount);
                }
            }
            
            // Show confirmation message
            let confirmation = format!(
                "✅ Added {} expense(s):\n{}",
                parsed_expenses.len(),
                parsed_expenses
                    .iter()
                    .map(|(desc, amount)| format!("• {} - {:.2}", desc, amount))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            
            bot.send_message(msg.chat.id, confirmation).await?;
            
            // Show updated expenses list
            let expenses_list = {
                let expenses = storage.lock().unwrap();
                format_expenses_list(&expenses)
            };
            
            bot.send_message(msg.chat.id, expenses_list).await?;
        } else if text.starts_with("/list") {
            // Show current expenses list
            let expenses_list = {
                let expenses = storage.lock().unwrap();
                format_expenses_list(&expenses)
            };
            
            bot.send_message(msg.chat.id, expenses_list).await?;
        } else if text.starts_with("/clear") {
            // Clear all expenses
            {
                let mut expenses = storage.lock().unwrap();
                expenses.clear();
            }
            
            bot.send_message(msg.chat.id, "🗑️ All expenses cleared!").await?;
        } else if text.starts_with("/help") || text.starts_with("/start") {
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
                `/help` - Show this help";
            
            bot.send_message(msg.chat.id, help_text).await?;
        } else {
            // If no expenses found in the message, provide helpful feedback
            bot.send_message(
                msg.chat.id, 
                "No expenses found in this message. Please use format:\n`<description> <amount>`\n\nExample: `Coffee 5.50`"
            ).await?;
        }
    } else {
        bot.send_message(
            msg.chat.id, 
            "Please send text messages with expense information in format:\n`<description> <amount>`"
        ).await?;
    }
    
    Ok(())
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

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let storage = storage.clone();
        async move {
            handle_message(bot, msg, storage).await
        }
    })
    .await;
}
