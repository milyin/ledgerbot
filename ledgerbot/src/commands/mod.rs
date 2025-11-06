pub mod command_add_category;
pub mod command_add_expense;
pub mod command_add_filter;
pub mod command_add_share;
pub mod command_add_words_filter;
pub mod command_categories;
pub mod command_clear_categories;
pub mod command_clear_expenses;
pub mod command_edit_filter;
pub mod command_edit_words_filter;
pub mod command_follow;
pub mod command_help;
pub mod command_list;
pub mod command_list_shares;
pub mod command_remove_category;
pub mod command_remove_filter;
pub mod command_remove_share;
pub mod command_rename_category;
pub mod command_report;
pub mod command_select_period;
pub mod command_start;
pub mod command_unfollow;
pub mod expenses;
pub mod follow_helper;
pub mod report;

use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{Chat, MessageId},
    utils::command::BotCommands,
};
use yoroolbot::command_trait::{CommandReplyTarget, CommandTrait};

use crate::{
    commands::{
        command_add_category::CommandAddCategory, command_add_expense::CommandAddExpense,
        command_add_filter::CommandAddFilter, command_add_share::CommandAddShare,
        command_add_words_filter::CommandAddWordsFilter, command_categories::CommandCategories,
        command_clear_categories::CommandClearCategories,
        command_clear_expenses::CommandClearExpenses, command_edit_filter::CommandEditFilter,
        command_edit_words_filter::CommandEditWordsFilter, command_follow::CommandFollow,
        command_help::CommandHelp, command_list::CommandList,
        command_list_shares::CommandListShares, command_remove_category::CommandRemoveCategory,
        command_remove_filter::CommandRemoveFilter, command_remove_share::CommandRemoveShare,
        command_rename_category::CommandRenameCategory, command_report::CommandReport,
        command_select_period::CommandSelectPeriod, command_start::CommandStart,
        command_unfollow::CommandUnfollow,
    },
    storages::StorageTrait,
};

/// Bot commands
#[derive(BotCommands, Clone, Debug, PartialEq)]
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
        description = "add username to share list",
        rename = "add_share",
        parse_with = CommandAddShare::parse_arguments
    )]
    AddShare(CommandAddShare),
    #[command(
        description = "list all usernames in share list",
        rename = "list_shares",
        parse_with = CommandListShares::parse_arguments
    )]
    ListShares(CommandListShares),
    #[command(
        description = "remove username from share list",
        rename = "remove_share",
        parse_with = CommandRemoveShare::parse_arguments
    )]
    RemoveShare(CommandRemoveShare),
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
}

impl From<Command> for String {
    fn from(val: Command) -> Self {
        match val {
            Command::Start(start) => start.to_command_string(true),
            Command::Help(help) => help.to_command_string(true),
            Command::List(list) => list.to_command_string(true),
            Command::Report(report) => report.to_command_string(true),
            Command::ClearExpenses(clear_expenses) => clear_expenses.to_command_string(true),
            Command::Categories(categories) => categories.to_command_string(true),
            Command::ClearCategories(clear_categories) => clear_categories.to_command_string(true),
            Command::AddCategory(add_category) => add_category.to_command_string(true),
            Command::AddFilter(add_filter) => add_filter.to_command_string(true),
            Command::RemoveCategory(remove_category) => remove_category.to_command_string(true),
            Command::RenameCategory(rename_category) => rename_category.to_command_string(true),
            Command::RemoveFilter(remove_filter) => remove_filter.to_command_string(true),
            Command::EditFilter(edit_filter) => edit_filter.to_command_string(true),
            Command::AddExpense(add_expense) => add_expense.to_command_string(true),
            Command::AddWordsFilter(add_words_filter) => add_words_filter.to_command_string(true),
            Command::EditWordsFilter(edit_words_filter) => {
                edit_words_filter.to_command_string(true)
            }
            Command::SelectPeriod(select_period) => select_period.to_command_string(true),
            Command::AddShare(add_share) => add_share.to_command_string(true),
            Command::ListShares(list_shares) => list_shares.to_command_string(true),
            Command::RemoveShare(remove_share) => remove_share.to_command_string(true),
            Command::Follow(follow) => follow.to_command_string(true),
            Command::Unfollow(unfollow) => unfollow.to_command_string(true),
        }
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
    storage: Arc<dyn StorageTrait>,
    cmd: Command,
    batch: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target = CommandReplyTarget {
        bot: bot.clone(),
        chat: chat.clone(),
        msg_id,
        batch,
        callback_data_storage: storage.clone().as_callback_data_storage(),
    };
    match cmd {
        Command::Start(start) => {
            start.run(&target, ()).await?;
        }
        Command::Help(help) => {
            help.run(&target, ()).await?;
        }
        Command::List(list) => {
            list.run(&target, storage.clone()).await?;
        }
        Command::Report(report) => {
            report.run(&target, storage.clone()).await?;
        }
        Command::ClearExpenses(clear_expenses) => {
            clear_expenses.run(&target, storage.clone()).await?;
        }
        Command::ClearCategories(clear_categories) => {
            clear_categories
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::AddCategory(add_category) => {
            add_category
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::Categories(categories) => {
            categories.run(&target, storage.clone()).await?;
        }
        Command::AddFilter(add_filter) => {
            add_filter.run(&target, storage.clone()).await?;
        }
        Command::RemoveCategory(remove_category) => {
            remove_category
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::RenameCategory(rename_category) => {
            rename_category
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::RemoveFilter(remove_filter) => {
            remove_filter
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::EditFilter(edit_filter) => {
            edit_filter
                .run(&target, storage.clone().as_category_storage())
                .await?;
        }
        Command::AddExpense(add_expense) => {
            add_expense.run(&target, storage.clone()).await?;
        }
        Command::AddWordsFilter(add_words_filter) => {
            add_words_filter.run(&target, storage.clone()).await?;
        }
        Command::EditWordsFilter(edit_words_filter) => {
            edit_words_filter.run(&target, storage.clone()).await?;
        }
        Command::SelectPeriod(select_period) => {
            select_period.run(&target, storage.clone()).await?;
        }
        Command::AddShare(add_share) => {
            add_share
                .run(&target, storage.clone().as_share_storage())
                .await?;
        }
        Command::ListShares(list_shares) => {
            list_shares
                .run(&target, storage.clone().as_share_storage())
                .await?;
        }
        Command::RemoveShare(remove_share) => {
            remove_share
                .run(&target, storage.clone().as_share_storage())
                .await?;
        }
        Command::Follow(follow) => {
            follow.run(&target, storage.clone()).await?;
        }
        Command::Unfollow(unfollow) => {
            unfollow.run(&target, storage.clone()).await?;
        }
    }
    Ok(())
}
