use std::{collections::HashMap, sync::Arc};

use teloxide::prelude::ResponseResult;
use yoroolbot::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
    markdown_string, storage::ButtonData,
};

use crate::storage_traits::CategoryStorageTrait;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandClearCategories {
    pub confirm: Option<bool>,
}

impl CommandTrait for CommandClearCategories {
    type A = bool;
    type B = EmptyArg;
    type C = EmptyArg;
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    type Context = Arc<dyn CategoryStorageTrait>;

    const NAME: &'static str = "clear_categories";
    const PLACEHOLDERS: &[&'static str] = &["<confirm>"];

    fn param1(&self) -> Option<&Self::A> {
        self.confirm.as_ref()
    }

    fn from_arguments(
        confirm: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandClearCategories { confirm }
    }

    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _storage: Self::Context,
    ) -> ResponseResult<()> {
        // Show confirmation prompt with buttons
        let message = markdown_string!("🗑️ Confirm clearing all categories\\?");

        let buttons = vec![
            vec![
                ButtonData::Callback(
                    "✅ Yes, Clear All".to_string(),
                    CommandClearCategories {
                        confirm: Some(true),
                    }
                    .to_command_string(false),
                ),
                ButtonData::Callback(
                    "❌ Cancel".to_string(),
                    CommandClearCategories {
                        confirm: Some(false),
                    }
                    .to_command_string(false),
                ),
            ],
        ];

        target.markdown_message_with_menu(message, buttons).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        storage: Self::Context,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_string!("❌ Clear categories cancelled\\."))
                .await?;
            return Ok(());
        }

        if let Err(e) = storage
            .replace_categories(target.chat.id, HashMap::new())
            .await
        {
            target.send_markdown_message(e).await?;
            return Ok(());
        }

        target
            .send_markdown_message(markdown_string!("🗑️ All categories cleared\\!"))
            .await?;
        Ok(())
    }
}

impl From<CommandClearCategories> for crate::commands::Command {
    fn from(cmd: CommandClearCategories) -> Self {
        crate::commands::Command::ClearCategories(cmd)
    }
}
