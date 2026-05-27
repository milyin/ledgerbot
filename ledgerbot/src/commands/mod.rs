pub mod command_add_category;
pub mod command_add_expense;
pub mod command_add_filter;
pub mod command_add_follower;
pub mod command_add_words_filter;
pub mod command_categories;
pub mod command_clear_categories;
pub mod command_clear_expenses;
pub mod command_edit_filter;
pub mod command_edit_words_filter;
pub mod command_follow;
pub mod command_help;
pub mod command_list;
pub mod command_list_followers;
pub mod command_remove_category;
pub mod command_remove_filter;
pub mod command_remove_follower;
pub mod command_rename_category;
pub mod command_report;
pub mod command_select_period;
pub mod command_start;
pub mod command_unfollow;
pub mod expenses;
pub mod follow_helper;
pub mod report;
pub mod support;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use telluride::command::CallbackKey;
use telluride::data_store::InMemStore;
use teloxide::{
    prelude::*,
    types::{Chat, MessageId, UserId},
    utils::command::BotCommands,
};

use crate::{
    commands::{
        command_add_category::CommandAddCategory, command_add_expense::CommandAddExpense,
        command_add_filter::CommandAddFilter, command_add_follower::CommandAddFollower,
        command_add_words_filter::CommandAddWordsFilter, command_categories::CommandCategories,
        command_clear_categories::CommandClearCategories,
        command_clear_expenses::CommandClearExpenses, command_edit_filter::CommandEditFilter,
        command_edit_words_filter::CommandEditWordsFilter, command_follow::CommandFollow,
        command_help::CommandHelp, command_list::CommandList,
        command_list_followers::CommandListFollowers,
        command_remove_category::CommandRemoveCategory, command_remove_filter::CommandRemoveFilter,
        command_remove_follower::CommandRemoveFollower,
        command_rename_category::CommandRenameCategory, command_report::CommandReport,
        command_select_period::CommandSelectPeriod, command_start::CommandStart,
        command_unfollow::CommandUnfollow, follow_helper::validate_and_get_follow_access,
    },
    storages::Stores,
    ui::CommandContext,
};

/// Bot commands
#[derive(BotCommands, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(
        description = "start the bot",
        parse_with = CommandStart::parse_arguments
    )]
    Start(CommandStart),
    #[command(
        description = "display this help",
        parse_with = CommandHelp::parse_arguments
    )]
    Help(CommandHelp),
    #[command(
        description = "list expenses chronologically in input format",
        parse_with = CommandList::parse_arguments
    )]
    List(CommandList),
    #[command(
        description = "show expenses report",
        parse_with = CommandReport::parse_arguments
    )]
    Report(CommandReport),
    #[command(
        description = "clear all expenses",
        rename = "clear_expenses",
        parse_with = CommandClearExpenses::parse_arguments
    )]
    ClearExpenses(CommandClearExpenses),
    #[command(
        description = "list all categories with filters in command format",
        parse_with = CommandCategories::parse_arguments
    )]
    Categories(CommandCategories),
    #[command(
        description = "clear all categories",
        rename = "clear_categories",
        parse_with = CommandClearCategories::parse_arguments
    )]
    ClearCategories(CommandClearCategories),
    #[command(
        description = "add expense category",
        rename = "add_category",
        parse_with = CommandAddCategory::parse_arguments
    )]
    AddCategory(CommandAddCategory),
    #[command(
        description = "add filter to category",
        rename = "add_filter",
        parse_with = CommandAddFilter::parse_arguments
    )]
    AddFilter(CommandAddFilter),
    #[command(
        description = "remove expense category",
        rename = "remove_category",
        parse_with = CommandRemoveCategory::parse_arguments
    )]
    RemoveCategory(CommandRemoveCategory),
    #[command(
        description = "rename expense category",
        rename = "rename_category",
        parse_with = CommandRenameCategory::parse_arguments
    )]
    RenameCategory(CommandRenameCategory),
    #[command(
        description = "remove filter from category by position",
        rename = "remove_filter",
        parse_with = CommandRemoveFilter::parse_arguments
    )]
    RemoveFilter(CommandRemoveFilter),
    #[command(
        description = "edit filter in category by position",
        rename = "edit_filter",
        parse_with = CommandEditFilter::parse_arguments
    )]
    EditFilter(CommandEditFilter),
    #[command(
        description = "add expense with explicit date, description and amount",
        rename = "add_expense",
        parse_with = CommandAddExpense::parse_arguments
    )]
    AddExpense(CommandAddExpense),
    #[command(
        description = "add new word-based filter to category",
        rename = "add_words_filter",
        parse_with = CommandAddWordsFilter::parse_arguments
    )]
    AddWordsFilter(CommandAddWordsFilter),
    #[command(
        description = "edit word-based filter in category by position",
        rename = "edit_words_filter",
        parse_with = CommandEditWordsFilter::parse_arguments
    )]
    EditWordsFilter(CommandEditWordsFilter),
    #[command(
        description = "select period for expense tracking",
        rename = "select_period",
        parse_with = CommandSelectPeriod::parse_arguments
    )]
    SelectPeriod(CommandSelectPeriod),
    #[command(
        description = "add username to followers list",
        rename = "add_follower",
        parse_with = CommandAddFollower::parse_arguments
    )]
    AddFollower(CommandAddFollower),
    #[command(
        description = "list all usernames in followers list",
        rename = "list_followers",
        parse_with = CommandListFollowers::parse_arguments
    )]
    ListFollowers(CommandListFollowers),
    #[command(
        description = "remove username from followers list",
        rename = "remove_follower",
        parse_with = CommandRemoveFollower::parse_arguments
    )]
    RemoveFollower(CommandRemoveFollower),
    #[command(
        description = "follow expenses from another chat",
        parse_with = CommandFollow::parse_arguments
    )]
    Follow(CommandFollow),
    #[command(
        description = "stop following another chat",
        parse_with = CommandUnfollow::parse_arguments
    )]
    Unfollow(CommandUnfollow),
}

// Command constants as string representations
impl Command {
    pub const ADD_FILTER: &'static str = "/add_filter";

    pub fn to_command_string(&self, complete: bool) -> String {
        match self {
            Command::Start(start) => start.to_command_string(complete),
            Command::Help(help) => help.to_command_string(complete),
            Command::List(list) => list.to_command_string(complete),
            Command::Report(report) => report.to_command_string(complete),
            Command::ClearExpenses(clear_expenses) => clear_expenses.to_command_string(complete),
            Command::Categories(categories) => categories.to_command_string(complete),
            Command::ClearCategories(clear_categories) => {
                clear_categories.to_command_string(complete)
            }
            Command::AddCategory(add_category) => add_category.to_command_string(complete),
            Command::AddFilter(add_filter) => add_filter.to_command_string(complete),
            Command::RemoveCategory(remove_category) => remove_category.to_command_string(complete),
            Command::RenameCategory(rename_category) => rename_category.to_command_string(complete),
            Command::RemoveFilter(remove_filter) => remove_filter.to_command_string(complete),
            Command::EditFilter(edit_filter) => edit_filter.to_command_string(complete),
            Command::AddExpense(add_expense) => add_expense.to_command_string(complete),
            Command::AddWordsFilter(add_words_filter) => {
                add_words_filter.to_command_string(complete)
            }
            Command::EditWordsFilter(edit_words_filter) => {
                edit_words_filter.to_command_string(complete)
            }
            Command::SelectPeriod(select_period) => select_period.to_command_string(complete),
            Command::AddFollower(add_follower) => add_follower.to_command_string(complete),
            Command::ListFollowers(list_followers) => list_followers.to_command_string(complete),
            Command::RemoveFollower(remove_follower) => remove_follower.to_command_string(complete),
            Command::Follow(follow) => follow.to_command_string(complete),
            Command::Unfollow(unfollow) => unfollow.to_command_string(complete),
        }
    }
}

impl From<Command> for String {
    fn from(val: Command) -> Self {
        val.to_command_string(true)
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

/// Execute a single command (helper function for batch processing and text message handling)
pub async fn execute_command(
    bot: Bot,
    chat: Chat,
    msg_id: Option<MessageId>,
    user_id: UserId,
    stores: Arc<Stores>,
    callback_storage: Arc<InMemStore<CallbackKey, Command>>,
    cmd: Command,
    batch: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let storage = stores.storage(chat.id);
    let target = CommandContext {
        bot: bot.clone(),
        chat: chat.clone(),
        message_id: msg_id,
        user_id,
        batch,
        callback_storage,
    };
    // Validate follow access (setting to use data from another user) and
    // get appropriate readonly storage. The readwrite storage is always for the current chat.
    let storage_readonly = match validate_and_get_follow_access(&target, &stores).await {
        Ok(storage) => storage,
        Err(warning_msg) => {
            target.send_markdown_message(warning_msg).await?;
            // Continue with current chat
            stores.storage_readonly(target.chat.id, target.chat.id)
        }
    };

    match cmd {
        Command::Start(start) => {
            start.execute(&target, ()).await?;
        }
        Command::Help(help) => {
            help.execute(&target, ()).await?;
        }
        Command::List(list) => {
            list.execute(&target, storage_readonly).await?;
        }
        Command::Report(report) => {
            report.execute(&target, storage_readonly).await?;
        }
        Command::ClearExpenses(clear_expenses) => {
            clear_expenses.execute(&target, storage).await?;
        }
        Command::ClearCategories(clear_categories) => {
            clear_categories
                .execute(&target, storage.categories())
                .await?;
        }
        Command::AddCategory(add_category) => {
            add_category.execute(&target, storage.categories()).await?;
        }
        Command::Categories(categories) => {
            categories.execute(&target, storage_readonly).await?;
        }
        Command::AddFilter(add_filter) => {
            add_filter.execute(&target, storage).await?;
        }
        Command::RemoveCategory(remove_category) => {
            remove_category
                .execute(&target, stores.storage(target.chat.id).categories())
                .await?;
        }
        Command::RenameCategory(rename_category) => {
            rename_category
                .execute(&target, storage.categories())
                .await?;
        }
        Command::RemoveFilter(remove_filter) => {
            remove_filter.execute(&target, storage.categories()).await?;
        }
        Command::EditFilter(edit_filter) => {
            edit_filter.execute(&target, storage.categories()).await?;
        }
        Command::AddExpense(add_expense) => {
            add_expense.execute(&target, storage).await?;
        }
        Command::AddWordsFilter(add_words_filter) => {
            add_words_filter.execute(&target, storage).await?;
        }
        Command::EditWordsFilter(edit_words_filter) => {
            edit_words_filter.execute(&target, stores.clone()).await?;
        }
        Command::SelectPeriod(select_period) => {
            select_period.execute(&target, storage).await?;
        }
        Command::AddFollower(add_follower) => {
            add_follower
                .execute(&target, stores.storage(chat.id).followers())
                .await?;
        }
        Command::ListFollowers(list_followers) => {
            list_followers
                .execute(&target, stores.storage(chat.id).followers())
                .await?;
        }
        Command::RemoveFollower(remove_follower) => {
            remove_follower
                .execute(&target, stores.storage(chat.id).followers())
                .await?;
        }
        Command::Follow(follow) => {
            follow.execute(&target, stores.clone()).await?;
        }
        Command::Unfollow(unfollow) => {
            unfollow.execute(&target, stores.clone()).await?;
        }
    }
    Ok(())
}
