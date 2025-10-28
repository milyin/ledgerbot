---
name: command-trait
description: Use this skill when implementing new Telegram bot commands using CommandTrait, modifying existing commands, or working with command parameters, execution methods (run0, run1, etc.), CommandReplyTarget, MarkdownStringMessage, ButtonData, callback data storage, or the progressive parameter gathering pattern. Also invoke for questions about command architecture, registration, or message sending methods.
---

# CommandTrait Architecture

This skill provides documentation on the CommandTrait-based command architecture used in the ledgerbot project.

## Overview

The ledgerbot uses a trait-based architecture for implementing Telegram bot commands. The core framework is located in the **yoroolbot** library crate, which provides reusable components for Telegram bot development. Commands that implement `CommandTrait` get automatic parsing, validation, and execution capabilities.

## Project Structure

- **yoroolbot/** - Library crate containing the bot framework
  - `src/api/command_trait/mod.rs` - CommandTrait definition and CommandReplyTarget
  - `src/api/markdown/` - MarkdownString utilities
  - `src/api/storage/callback_data_storage.rs` - Callback data packing/unpacking
- **ledgerbot/** - Main bot application (binary crate)
  - `src/commands/` - Command implementations
  - `src/menus/` - Interactive menu helpers
  - `src/storage_traits.rs` - Storage trait definitions

## Core Components

### 1. CommandTrait

**Location**: `yoroolbot/src/api/command_trait/mod.rs`

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

**Location**: `yoroolbot/src/api/command_trait/mod.rs`

A wrapper struct that encapsulates the context for command execution:

**Fields:**
- `bot`: The Telegram Bot instance
- `chat`: The chat where the command was invoked
- `msg_id`: Optional message ID (used for editing or replying to specific messages)
- `batch`: Boolean flag for batch processing mode
- `callback_data_storage`: Arc to callback data storage for handling long button data

**Key methods**:

- `markdown_message(&self, text: MarkdownString) -> ResponseResult<Message>`
  - Smart message handler: If `msg_id` is `Some(id)`, edits the existing message. If `msg_id` is `None`, sends a new message.
  - Use this for navigation and prompts

- `markdown_message_with_menu<R, B>(&self, text: MarkdownString, menu: impl IntoIterator<Item = R>) -> ResponseResult<Message>`
  - Sends a markdown message with an inline keyboard menu
  - Automatically packs callback data using `pack_callback_data`
  - Supports both callback buttons and inline query buttons via `ButtonData` enum

- `send_markdown_message(&self, text: MarkdownString) -> JsonRequest<SendMessage>`
  - Always sends a new message
  - Returns a request builder for customization
  - Use this for results and errors

- `send_markdown_message_with_menu<R, B>(&self, text: MarkdownString, menu: impl IntoIterator<Item = R>) -> ResponseResult<Message>`
  - Sends a new message with an inline keyboard menu
  - Automatically packs callback data

- `edit_markdown_message_text(&self, message_id: MessageId, text: MarkdownString) -> EditMessageText`
  - Edits a specific message by ID
  - Use this for targeted message updates

### 3. Callback Data Storage

**Location**: `yoroolbot/src/api/storage/callback_data_storage.rs`

Telegram limits callback data to 64 bytes. The callback data storage system solves this by:
- Storing long callback data in memory with short reference strings (e.g., "cb:chat123:msg456:btn0")
- Automatically packing/unpacking data when buttons are created/clicked
- Managing lifecycle of stored data per message

**Key components:**

- `CallbackDataStorageTrait` - Async trait for storage operations
- `CallbackDataStorage` - In-memory implementation using `Arc<Mutex<HashMap>>`
- `pack_callback_data()` - Converts button data to keyboard, storing long data
- `unpack_callback_data()` - Retrieves original data from storage references
- `ButtonData` enum - Supports both callback and inline query buttons

**ButtonData enum**:
```rust
pub enum ButtonData {
    Callback(String, String),         // (label, callback_data)
    SwitchInlineQuery(String, String), // (label, query_text)
}
```

This allows mixing different button types in the same menu while automatically handling storage packing for callback buttons.

**Usage pattern**:
- Commands use `markdown_message_with_menu()` or `send_markdown_message_with_menu()`
- Pass menu data as `Vec<Vec<ButtonData>>` or anything that converts to it via `Into<ButtonData>`
- Framework automatically packs long callback data into storage
- When button is clicked, framework automatically unpacks the original data

For implementation details, see the actual source files rather than code examples here.

### 4. MarkdownStringMessage Trait

**Location**: `yoroolbot/src/api/markdown/string.rs`

This trait extends `teloxide::Bot` with methods that accept `MarkdownString` and automatically set `ParseMode::MarkdownV2`.

**Trait methods:**
- `markdown_message(chat_id, message_id: Option<MessageId>, text)` - Smart send/edit
- `send_markdown_message(chat_id, text)` - Always sends new message
- `edit_markdown_message_text(chat_id, message_id, text)` - Always edits existing message

See the source file for implementation details.

### 5. EmptyArg

**Location**: `yoroolbot/src/api/command_trait/mod.rs`

A marker type used for unused command parameters. Implements `ParseCommandArg` and always expects an empty string.

## File Naming Convention

Commands follow this pattern:
- File: `command_<name>.rs` (e.g., `command_help.rs`, `command_report.rs`)
- Struct: `Command<Name>` (e.g., `CommandHelp`, `CommandReport`)
- Location: `ledgerbot/src/commands/`

## Implementation Pattern

### Creating a New Command

See existing command implementations for the current pattern:
- Simple commands (no parameters): `ledgerbot/src/commands/command_help.rs`
- Single parameter: `ledgerbot/src/commands/command_add_category.rs`
- Multiple parameters: `ledgerbot/src/commands/command_remove_filter.rs`
- Complex interactive: `ledgerbot/src/commands/command_add_words_filter.rs`

Key steps:
1. Create `command_<name>.rs` with struct implementing `CommandTrait`
2. Register in `ledgerbot/src/commands/mod.rs`:
   - Add module declaration
   - Add to `Command` enum with teloxide attributes
   - Update `From<Command> for String`
   - Update `execute_command()` match arm

### Command Structure Requirements

**Type definitions:**
```rust
type A = FirstParamType;  // or EmptyArg if unused
type B = SecondParamType; // or EmptyArg if unused
// ... up to type I
type Context = Arc<dyn SomeTrait>; // or () if no context needed
```

**Constants:**
```rust
const NAME: &'static str = "command_name";
const PLACEHOLDERS: &[&'static str] = &["<param1>", "<param2>"]; // or &[] if no params
```

**Required methods:**
```rust
fn from_arguments(...) -> Self { /* construct from Option<T> parameters */ }
fn param1(&self) -> Option<&Self::A> { /* return reference to field */ }
fn param2(&self) -> Option<&Self::B> { /* return reference to field */ }
// ... one paramN() for each non-EmptyArg parameter

async fn run0(&self, target: &CommandReplyTarget, context: Self::Context) -> ResponseResult<()>
// ... and run1, run2, etc. as needed
```

**CRITICAL**: You MUST implement `paramN()` methods for each parameter. Without these, the trait cannot dispatch to the correct `runN` method.

## Progressive Parameter Gathering Pattern

The CommandTrait architecture supports a powerful pattern for commands that need to gather multiple parameters interactively.

### How It Works

The `run()` method (provided by the trait) automatically dispatches to the appropriate `runN` method based on how many parameters are present:
- **run0()** - Called when NO parameters are provided
- **run1(param1)** - Called when 1 parameter is provided
- **run2(param1, param2)** - Called when 2 parameters are provided
- And so on...

### Example Flow

For a command like `/remove_filter`:
1. User types `/remove_filter` → `run0()` shows category selection menu
2. User clicks "Food" → `/remove_filter Food` → `run1(category)` shows filter selection menu
3. User clicks position "0" → `/remove_filter Food 0` → `run2(category, position)` performs removal

### Menu Helper Functions

**Location**: `ledgerbot/src/menus/`

Key menu functions:
- `select_category()` - Category selection menu
- `select_category_filter()` - Filter selection within category
- `select_word()` - Word selection with pagination for filter creation
- Helper functions for validation and reading data

See the actual source files for function signatures and usage patterns.

### Key Principles

1. **Each runN method has a single responsibility**
2. **Use closure constructors for next commands** to create callback data
3. **Back navigation** via optional back commands
4. **Validation at each step** with early returns on failure
5. **Final action in the last runN**
6. **Support multiple usage modes**: Interactive (menus), Direct (all params), Partial (some params)

## Message Method Selection

**Rule of thumb:**
- **Navigation/Prompts** → `markdown_message()` (can edit in-place)
- **Results/Errors** → `send_markdown_message()` (always new message)
- **Specific edits** → `edit_markdown_message_text()` (by message ID)
- **With menus** → `markdown_message_with_menu()` or `send_markdown_message_with_menu()`

## Best Practices

1. **Context Type Selection**:
   - Use `()` if no storage needed
   - Use `Arc<dyn SpecificTrait>` for single storage type
   - Use `Arc<dyn StorageTrait>` for multiple storage types (provides `.as_expense_storage()`, etc.)

2. **Parameter Types**:
   - Use built-in types implementing `FromStr`
   - For custom types, implement `ParseCommandArg` trait
   - Use `EmptyArg` for unused slots

3. **Run Methods**:
   - Always implement `run0` for no-parameters case
   - Implement `run1`, `run2`, etc. based on parameter count
   - The trait's `run()` method dispatches automatically

4. **Parameter Accessors (CRITICAL)**:
   - **MUST implement `paramN()` methods** for each parameter
   - Return `Some(&self.field)` for the corresponding field
   - Without these, dispatch won't work

5. **Message Formatting**:
   - Always use `markdown_format!` or `markdown_string!` macros
   - These handle proper MarkdownV2 escaping
   - Never manually escape strings

6. **Error Handling**:
   - Use `ResponseResult<()>` as return type
   - Return errors with `?` operator
   - Send user-friendly errors with `send_markdown_message()`

7. **Using to_command_string()**:
   - Use `to_command_string(true)` for usage/help (shows placeholders)
   - Use `to_command_string(false)` for examples (shows actual values)
   - Never hardcode command strings - always use this method

8. **Callback Data**:
   - Use `ButtonData::Callback` for regular callback buttons
   - Use `ButtonData::SwitchInlineQuery` for buttons that put text in input box
   - The framework automatically handles storage packing
   - Long callback data is automatically stored and referenced

## Source of Truth

This documentation provides an overview. For implementation details, always refer to the actual source code:

**Framework (yoroolbot):**
- `yoroolbot/src/api/command_trait/mod.rs` - Core traits and types
- `yoroolbot/src/api/markdown/` - Markdown utilities
- `yoroolbot/src/api/storage/callback_data_storage.rs` - Callback data system

**Application (ledgerbot):**
- `ledgerbot/src/commands/` - Command implementations (examples)
- `ledgerbot/src/commands/mod.rs` - Command registration
- `ledgerbot/src/menus/` - Menu helpers
- `CLAUDE.md` - General project documentation

The codebase is the authoritative source. This skill provides conceptual guidance and points to the relevant files for implementation details.
