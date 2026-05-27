use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use telluride::markdown_string;
use teloxide::prelude::ResponseResult;

use crate::{
    impl_command_execute_1, impl_command_io,
    storages::CategoryStorageTrait,
    ui::{ButtonData, CommandContext},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandClearCategories {
    pub confirm: Option<bool>,
}

impl CommandClearCategories {
    async fn run0(
        &self,
        target: &CommandContext,
        _storage: Arc<dyn CategoryStorageTrait>,
    ) -> ResponseResult<()> {
        let message = markdown_string!("🗑️ Confirm clearing all categories\\?");
        let buttons = vec![vec![ButtonData::SwitchInlineQuery(
            "✅ Yes, Clear All".to_string(),
            CommandClearCategories {
                confirm: Some(true),
            }
            .to_command_string(false),
        )]];
        target.markdown_message_with_menu(message, buttons).await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandContext,
        storage: Arc<dyn CategoryStorageTrait>,
        confirm: &bool,
    ) -> ResponseResult<()> {
        if !*confirm {
            target
                .send_markdown_message(markdown_string!("❌ Clear categories cancelled\\."))
                .await?;
            return Ok(());
        }

        if let Err(e) = storage.replace_categories(HashMap::new()).await {
            target.send_markdown_message(e).await?;
            return Ok(());
        }

        target
            .send_markdown_message(markdown_string!("🗑️ All categories cleared\\!"))
            .await?;
        Ok(())
    }
}

impl_command_io!(CommandClearCategories, "clear_categories", ["<confirm>"], confirm: bool);
impl_command_execute_1!(
    CommandClearCategories,
    Arc<dyn CategoryStorageTrait>,
    confirm
);

impl From<CommandClearCategories> for crate::commands::Command {
    fn from(cmd: CommandClearCategories) -> Self {
        crate::commands::Command::ClearCategories(cmd)
    }
}
