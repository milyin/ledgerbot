# CommandTrait Architecture

This skill provides documentation on the CommandTrait-based command architecture used in the ledgerbot project.

## Overview

The ledgerbot uses a trait-based architecture for implementing Telegram bot commands. Commands that implement `CommandTrait` get automatic parsing, validation, and execution capabilities.

## Core Components

### 1. CommandTrait

Located in: `ledgerbot/src/commands/command_trait.rs`

The `CommandTrait` provides:
- Up to 9 typed parameters (A through I)
- Automatic command parsing from strings
- Execution methods based on the number of provided arguments (`run0`, `run1`, ... `run9`)
- Command string generation with placeholder support
- Type-safe parameter handling

**Key associated types:**
- `A` through `I`: Parameter types (use `EmptyArg` for unused parameters)
- `Context`: The context type needed for execution (e.g., storage, `()` for no context)

**Key constants:**
- `NAME`: The command name (e.g., "help", "report")
- `PLACEHOLDERS`: Array of placeholder strings for parameters (e.g., `&["<name>"]`)

### 2. CommandReplyTarget

A wrapper struct that encapsulates the context for command execution:

**Fields:**
- `bot`: The Telegram Bot instance
- `chat`: The chat where the command was invoked
- `msg_id`: Optional message ID (used for editing or replying to specific messages)

**Helper methods** (these are convenience wrappers around the `MarkdownStringMessage` trait):

- `markdown_message(&self, text: MarkdownString) -> ResponseResult<Message>`
  - **Wrapper around** `bot.markdown_message(chat.id, msg_id, text)`
  - **Smart message handler**: If `msg_id` is `Some(id)`, edits the existing message with that ID. If `msg_id` is `None`, sends a new message.
  - Returns the message after sending/editing
  - Use this when you want automatic behavior based on context
  - **Most common choice** in command implementations

- `send_markdown_message(&self, text: MarkdownString) -> JsonRequest<SendMessage>`
  - **Wrapper around** `bot.send_markdown_message(chat.id, text)`
  - **Always sends a new message**, regardless of `msg_id`
  - Returns a request builder that can be further customized (e.g., `.reply_markup()`)
  - Must call `.await?` to execute
  - Use this when you need to customize the message or always want a new message

- `edit_markdown_message_text(&self, message_id: MessageId, text: MarkdownString) -> EditMessageText`
  - **Wrapper around** `bot.edit_markdown_message_text(chat.id, message_id, text)`
  - **Edits a specific message** by providing a message ID
  - The message ID doesn't have to be the one in `msg_id`
  - Returns a request builder that can be further customized
  - Must call `.await?` to execute
  - Use this when you need to edit a specific message

**Important Note:** These methods are convenience wrappers that automatically use `self.bot` and `self.chat.id` from the target. They delegate to the `MarkdownStringMessage` trait which is implemented for `Bot`.

### 3. MarkdownStringMessage Trait

Located in: `yoroolbot/src/api/markdown/string.rs`

This trait extends `teloxide::Bot` with methods that accept `MarkdownString` and automatically set `ParseMode::MarkdownV2`. The trait is implemented for `Bot` and provides the underlying functionality that `CommandReplyTarget` wraps.

**Trait methods:**
- `markdown_message(chat_id, message_id: Option<MessageId>, text)` - Smart send/edit
- `send_markdown_message(chat_id, text)` - Always sends new message
- `edit_markdown_message_text(chat_id, message_id, text)` - Always edits existing message

**Key difference from CommandReplyTarget:**
- **Trait methods** require explicit `chat_id` parameter
- **CommandReplyTarget methods** use `self.chat.id` automatically from the target context
- Both ultimately call the same trait implementation, but CommandReplyTarget provides a more convenient API within command handlers

### 4. EmptyArg

A marker type used for unused command parameters. Implements `ParseCommandArg` and always expects an empty string.

## Understanding the Two Layers

The messaging system has two layers:

1. **MarkdownStringMessage trait** (low-level) - Methods on `Bot` that require explicit `chat_id`
2. **CommandReplyTarget** (high-level) - Convenience wrapper that captures context

**Comparison:**

| Operation | Via MarkdownStringMessage Trait | Via CommandReplyTarget |
|-----------|--------------------------------|------------------------|
| Send new message | `bot.send_markdown_message(chat_id, text).await?` | `target.send_markdown_message(text).await?` |
| Smart send/edit | `bot.markdown_message(chat_id, msg_id, text).await?` | `target.markdown_message(text).await?` |
| Edit message | `bot.edit_markdown_message_text(chat_id, msg_id, text).await?` | `target.edit_markdown_message_text(msg_id, text).await?` |

**In command implementations**, always use `CommandReplyTarget` methods (right column) because:
- Less verbose (no need to pass `chat_id` every time)
- Context is automatically captured
- The `target` parameter is provided by `CommandTrait`

**Outside of commands** (e.g., in standalone bot handlers), use the trait methods directly on `bot`.

## Method Usage Examples

### When to use each CommandReplyTarget method:

**Use `markdown_message()` for navigation and interactive flows:**
```rust
// Use for menus, prompts, and interactive navigation
// This allows the bot to edit the same message when user navigates
// Examples: displaying parameter selection menus, category lists, etc.

async fn run0(&self, target: &CommandReplyTarget, storage: Self::Context) {
    // Show interactive menu - will edit if coming from callback
    target
        .markdown_message(markdown_string!("Select a category:"))
        .await?;
    // Display category selection buttons...
}
```

**Use `send_markdown_message()` for results and errors:**
```rust
// Always send a NEW message for:
// - Action results (success/failure messages)
// - Error messages
// - Final outputs that should remain visible

async fn run1(&self, target: &CommandReplyTarget, storage: Self::Context, name: &String) {
    match storage.add_category(name).await {
        Ok(()) => {
            // Send result as a NEW message
            target
                .send_markdown_message(markdown_format!(
                    "✅ Category `{}` created",
                    name
                ))
                .await?;
        }
        Err(err) => {
            // Send error as a NEW message
            target
                .send_markdown_message(markdown_format!(
                    "❌ Error: {}",
                    &*err
                ))
                .await?;
        }
    }
}

// When you need to customize the message
target
    .send_markdown_message(markdown_string!("Hello!"))
    .reply_markup(keyboard)  // Can chain additional options
    .await?;
```

**Use `edit_markdown_message_text()` when:**
```rust
// You need to edit a specific message (like from a callback)
let msg = target.markdown_message(markdown_string!("Initial")).await?;
// ... later ...
target
    .edit_markdown_message_text(msg.id, markdown_string!("Updated"))
    .await?;
```

**Rule of thumb:**
- **Navigation/Prompts** → `markdown_message()` (can edit in-place during navigation)
- **Results/Errors** → `send_markdown_message()` (always creates new message for visibility)
- **Specific edits** → `edit_markdown_message_text()` (when you have a specific message ID to edit)

## File Naming Convention

Commands follow this pattern:
- File: `command_<name>.rs` (e.g., `command_help.rs`, `command_report.rs`)
- Struct: `Command<Name>` (e.g., `CommandHelp`, `CommandReport`)

## Implementation Pattern

### Step 1: Create Command Module

Create a new file `ledgerbot/src/commands/command_<name>.rs`:

```rust
use std::sync::Arc;
use teloxide::prelude::ResponseResult;

use crate::commands::{
    command_trait::{CommandReplyTarget, CommandTrait, EmptyArg},
};
use crate::storage_traits::SomeStorageTrait; // if needed

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandYourCommand {
    // Optional: Add fields for parameters
    pub param1: Option<String>,
}

impl CommandTrait for CommandYourCommand {
    // Define parameter types
    type A = String; // First parameter type
    type B = EmptyArg; // Unused parameter
    type C = EmptyArg; // Unused parameter
    type D = EmptyArg;
    type E = EmptyArg;
    type F = EmptyArg;
    type G = EmptyArg;
    type H = EmptyArg;
    type I = EmptyArg;

    // Define context type (use () if no context needed)
    type Context = Arc<dyn SomeStorageTrait>;

    const NAME: &'static str = "your_command";
    const PLACEHOLDERS: &[&'static str] = &["<param1>"];

    fn from_arguments(
        a: Option<Self::A>,
        _: Option<Self::B>,
        _: Option<Self::C>,
        _: Option<Self::D>,
        _: Option<Self::E>,
        _: Option<Self::F>,
        _: Option<Self::G>,
        _: Option<Self::H>,
        _: Option<Self::I>,
    ) -> Self {
        CommandYourCommand { param1: a }
    }

    fn param1(&self) -> Option<&Self::A> {
        self.param1.as_ref()
    }

    // Implement run0 for no parameters, run1 for one parameter, etc.
    async fn run0(
        &self,
        target: &CommandReplyTarget,
        _context: Self::Context,
    ) -> ResponseResult<()> {
        target
            .send_markdown_message(markdown_string!("No parameters provided"))
            .await?;
        Ok(())
    }

    async fn run1(
        &self,
        target: &CommandReplyTarget,
        context: Self::Context,
        param1: &String,
    ) -> ResponseResult<()> {
        // Implement command logic here
        target
            .send_markdown_message(markdown_format!("Got parameter: {}", param1))
            .await?;
        Ok(())
    }
}

impl From<CommandYourCommand> for crate::commands::Command {
    fn from(cmd: CommandYourCommand) -> Self {
        crate::commands::Command::YourCommand(cmd)
    }
}
```

### Step 2: Register in mod.rs

In `ledgerbot/src/commands/mod.rs`:

1. Add module declaration:
```rust
pub mod command_your_command;
```

2. Add to imports:
```rust
use crate::commands::{
    // ... other imports
    command_your_command::CommandYourCommand,
    // ...
};
```

3. Update `Command` enum:
```rust
pub enum Command {
    // ... other commands
    #[command(
        description = "description of your command",
        parse_with = CommandYourCommand::parse_arguments
    )]
    YourCommand(CommandYourCommand),
    // ...
}
```

4. Update `From<Command> for String`:
```rust
impl From<Command> for String {
    fn from(val: Command) -> Self {
        match val {
            // ... other commands
            Command::YourCommand(cmd) => cmd.to_command_string(true),
            // ...
        }
    }
}
```

5. Update `execute_command` function:
```rust
match cmd {
    // ... other commands
    Command::YourCommand(your_command) => {
        your_command
            .run(
                &CommandReplyTarget {
                    bot: bot.clone(),
                    chat: chat.clone(),
                    msg_id,
                },
                storage.clone().as_some_storage(), // or () if no context
            )
            .await?;
    }
    // ...
}
```

## Examples

### Commands with No Parameters

Examples: `CommandHelp`, `CommandStart`, `CommandReport`

- All type parameters A-I are `EmptyArg`
- `PLACEHOLDERS` is empty: `&[]`
- Only implement `run0`
- Context can be `()` or `Arc<dyn SomeTrait>` depending on needs

### Commands with One Parameter

Example: `CommandAddCategory`

```rust
type A = String;
type B = EmptyArg;
// ... rest are EmptyArg

const PLACEHOLDERS: &[&'static str] = &["<name>"];

// Implement param1() to return the parameter
fn param1(&self) -> Option<&Self::A> {
    self.name.as_ref()
}

// Implement both run0 (no params) and run1 (with param)
async fn run0(...) { /* handle interactive mode */ }
async fn run1(..., name: &String) { /* handle direct mode */ }
```

### Commands with Multiple Parameters

Example: `CommandEditFilter`

```rust
type A = String; // category
type B = usize;  // position
type C = String; // new_pattern
// ... rest are EmptyArg

const PLACEHOLDERS: &[&'static str] = &["<category>", "<position>", "<new_pattern>"];

// Implement param1(), param2(), param3()
// Implement run0, run1, run2, run3 as needed
```

## Best Practices

1. **Context Type Selection**:
   - Use `()` if no storage/context is needed
   - Use `Arc<dyn SpecificTrait>` if only one storage type is needed
   - Use `Arc<dyn StorageTrait>` if multiple storage types are needed (provides `.as_expense_storage()`, `.as_category_storage()`, etc.)

2. **Parameter Types**:
   - Use built-in types that implement `FromStr` (String, usize, i32, etc.)
   - For custom types, implement `ParseCommandArg` trait
   - Use `EmptyArg` for all unused parameter slots

3. **Run Methods**:
   - Always implement `run0` for the no-parameters case
   - Implement `run1`, `run2`, etc. based on how many parameters your command accepts
   - The trait's `run()` method automatically dispatches to the correct method based on provided parameters

4. **Choosing the Right Message Method**:
   - Use `markdown_message()` for **navigation and prompts** (menus, parameter selection, interactive flows)
     - Allows editing the same message during navigation
     - Creates a smoother user experience for interactive workflows
   - Use `send_markdown_message()` for **results and errors** (success/failure messages, final outputs)
     - Always creates a new message that stays visible in chat history
     - Important for visibility of actions performed
     - Required when you need to customize the message (e.g., add `.reply_markup()`)
   - Use `edit_markdown_message_text()` when you need to **edit a specific message** by its ID

5. **Error Handling**:
   - Use `ResponseResult<()>` as the return type
   - Return errors with `?` operator
   - Send user-friendly error messages using `send_markdown_message()` (always new message for visibility)

6. **Message Formatting**:
   - Always use `markdown_format!` or `markdown_string!` macros for messages
   - These macros handle proper escaping for Telegram's MarkdownV2 format
   - Never manually escape strings

7. **Clippy Warnings**:
   - Don't use `.clone()` on `Option<MessageId>` - it implements `Copy`
   - Use the value directly in `CommandReplyTarget`

## Migration from Old Pattern

If you have an old-style command function:

```rust
pub async fn old_command(bot: Bot, msg: Message, storage: Arc<dyn SomeTrait>) -> ResponseResult<()> {
    // implementation
}
```

Convert it to:

1. Create `command_old.rs` with `CommandOld` struct
2. Move logic to `run0` method
3. Use `target` and `storage` from trait parameters
4. Update enum and registration in `mod.rs`
5. Remove old function and imports

## See Also

- Example implementations: `command_help.rs`, `command_start.rs`, `command_report.rs`, `command_add_category.rs`
- Trait definition: `command_trait.rs`
- Command registration: `mod.rs`
