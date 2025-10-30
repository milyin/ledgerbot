use std::sync::Arc;

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg, NoopCommand},
    markdown_format, markdown_string,
};

use crate::{
    commands::command_edit_filter::CommandEditFilter,
    menus::{
        common::read_category_filter_by_index,
        select_category::select_category,
        select_category_filter::select_category_filter,
        select_word::{Words, select_word},
    },
    storages::storage_traits::StorageTrait,
    utils::extract_words::extract_and_merge_words,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandEditWordsFilter {
    pub category: Option<String>,
    pub position: Option<usize>,
    pub page: Option<usize>,
    pub words: Option<Words>,
}

impl CommandTrait for CommandEditWordsFilter {
    type A = String;
    type B = usize;
    type C = usize;
    type D = Words;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn StorageTrait>;

    const NAME: &'static str = "edit_words_filter";
    const PLACEHOLDERS: &[&'static str] = &["<category>", "<position>", "<page>", "<words>"];

    fn from_arguments(
        category: Option<Self::A>,
        position: Option<Self::B>,
        page: Option<Self::C>,
        words: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandEditWordsFilter {
            category,
            position,
            page,
            words,
        }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.category.as_ref()
    }

    fn param2(&self) -> Option<&Self::B> {
        self.position.as_ref()
    }

    fn param3(&self) -> Option<&Self::C> {
        self.page.as_ref()
    }

    fn param4(&self) -> Option<&Self::D> {
        self.words.as_ref()
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
    ) -> ResponseResult<()> {
        select_category(
            target,
            &storage.as_category_storage(),
            markdown_string!("✏️ Select Category to edit word filter"),
            |name| CommandEditWordsFilter {
                category: Some(name.to_string()),
                position: None,
                page: None,
                words: None,
            },
            None::<NoopCommand>,
        )
        .await
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
    ) -> ResponseResult<()> {
        select_category_filter(
            target,
            &storage.as_category_storage(),
            category,
            markdown_format!(
                "✏️ Select word\\-based filter to edit in category `{}`",
                category
            ),
            |idx, pattern| {
                // Only show word-based filters (those that can be parsed by Words::read_pattern)
                Words::read_pattern(pattern).map(|_| CommandEditWordsFilter {
                    category: Some(category.clone()),
                    position: Some(idx),
                    page: None,
                    words: None,
                })
            },
            Some(CommandEditWordsFilter::default()),
        )
        .await
    }

    async fn run2(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
        position: &usize,
    ) -> ResponseResult<()> {
        //
        // Prefill with words from old pattern only when runned with <category> and <position>
        //
        let Some(current_pattern) = read_category_filter_by_index(
            target,
            &storage.clone().as_category_storage(),
            category,
            *position,
            Some(CommandEditWordsFilter {
                category: Some(category.clone()),
                position: None,
                page: None,
                words: None,
            }),
        )
        .await?
        else {
            return Ok(());
        };

        let words = Words::read_pattern(&current_pattern).unwrap_or_default();

        // Navigate to next page
        self.run4(target, storage, category, position, &0, &words)
            .await
    }

    async fn run3(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
        position: &usize,
        page: &usize,
    ) -> ResponseResult<()> {
        //
        // When page is already selected and words are not provided, assume that current words list is empty
        //
        self.run4(target, storage, category, position, page, &Words::default())
            .await
    }

    async fn run4(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        category: &String,
        position: &usize,
        page: &usize,
        selected_words: &Words,
    ) -> ResponseResult<()> {
        let category = category.clone();
        let position = *position;

        let Some(current_pattern) = read_category_filter_by_index(
            target,
            &storage.clone().as_category_storage(),
            category.as_str(),
            position,
            Some(CommandEditWordsFilter {
                category: Some(category.clone()),
                position: None,
                page: None,
                words: None,
            }),
        )
        .await?
        else {
            return Ok(());
        };
        let words = extract_and_merge_words(
            &storage,
            target.chat.id,
            Words::read_pattern(&current_pattern),
        )
        .await;

        // Show word selection menu with pagination
        let prompt = |current_page: usize, total_pages: usize, total_words: usize| {
            markdown_format!(
                "✏️ Edit word filter **\\#{}** in category `{}`\n\n{}\n\nPage {}/{} \\({} words total\\)",
                position,
                &category,
                @raw if selected_words.as_ref().is_empty() { markdown_format!("_no words selected_") } else { markdown_format!("`{}`", selected_words.to_string()) },
                current_page,
                total_pages,
                total_words
            )
        };

        let word_command = |word: &str| {
            let mut new_words = selected_words.as_ref().clone();
            if new_words.contains(&word.to_string()) {
                new_words.retain(|w| w != word);
            } else {
                new_words.push(word.to_string());
            }
            CommandEditWordsFilter {
                category: Some(category.clone()),
                position: Some(position),
                page: Some(*page),
                words: Some(new_words.into()),
            }
        };

        let page_command = |page_num: usize| CommandEditWordsFilter {
            category: Some(category.clone()),
            position: Some(position),
            page: Some(page_num),
            words: Some(selected_words.clone()),
        };

        // Apply command will edit the existing filter
        let apply_command = CommandEditFilter {
            category: Some(category.clone()),
            position: Some(position),
            pattern: selected_words.build_pattern(),
        };

        select_word(
            target,
            prompt,
            words.as_ref(),
            selected_words.as_ref(),
            *page,
            word_command,
            page_command,
            apply_command,
            Some(CommandEditWordsFilter {
                category: Some(category.clone()),
                position: None,
                page: None,
                words: None,
            }),
        )
        .await
    }
}

impl From<CommandEditWordsFilter> for crate::commands::Command {
    fn from(cmd: CommandEditWordsFilter) -> Self {
        crate::commands::Command::EditWordsFilter(cmd)
    }
}
