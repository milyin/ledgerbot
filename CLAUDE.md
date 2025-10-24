# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ledgerbot is a Telegram expense tracking bot written in Rust. It allows users to track expenses, categorize them using regex patterns, and generate reports. The project is organized as a Cargo workspace with two main crates:

- **ledgerbot**: Main bot application (binary crate)
- **yoroolbot**: Shared library providing markdown utilities for Telegram MarkdownV2 formatting

## Build & Development Commands

### Building
```bash
# Standard build (native)
cargo build

# Cross-compile for Linux x86_64 (requires x86_64-unknown-linux-gnu target)
./build.sh
```

The `build.sh` script:
- Downloads and compiles OpenSSL 3.6.0 statically for cross-compilation
- Builds for `x86_64-unknown-linux-gnu` target
- Strips the binary
- Uses `CARGO_BUILD_FLAGS` environment variable (set to `--release` for release builds)

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test --package ledgerbot
cargo test --package yoroolbot

# Run specific test
cargo test test_name
```

### Running Locally
```bash
# Set up environment first (copy .env.example to .env and configure)
cp .env.example .env

# Run with debug logging
RUST_LOG=debug cargo run --package ledgerbot -- --persistent-storage
```

### Deployment
```bash
# Deploy and run (debug mode)
./run.sh

# Deploy as release and start systemd service
./release.sh
```

Both scripts require `.env` file with:
- `DEPLOY_USER`: SSH username
- `DEPLOY_HOST`: Remote server hostname
- `DEPLOY_PATH`: Deployment path for debug builds
- `DEPLOY_PATH_RELEASE`: Deployment path for release builds

## Architecture

### Storage System

The bot uses a trait-based storage architecture with multiple storage backends:

1. **Storage Traits** (`storage_traits.rs`): Defines async traits for different data types
   - `ExpenseStorageTrait`: Expense data per chat
   - `CategoryStorageTrait`: Category definitions per chat
   - `FilterSelectionStorageTrait`: Temporary filter word selections
   - `FilterPageStorageTrait`: Pagination state for filter browsing
   - `BatchStorageTrait`: Command batching during message processing
   - `StorageTrait`: Unified trait providing access to all storage types

2. **Storage Implementations** (`storage.rs`):
   - **In-Memory Storage**: Default, no persistence
   - **Persistent Category Storage**: Lazy-loads categories from YAML files (one file per chat ID)
   - All other storage types are in-memory only

3. **Main Storage** (`Storage` struct): Composition of all storage types, configurable via builder pattern

### Command Processing Pipeline

1. **Message Reception** (`handlers.rs`): All text messages flow through `handle_text_message`
2. **Parsing** (`parser.rs`): `parse_expenses()` converts message text to `Vec<Result<Command, String>>`
   - Lines starting with `/` are parsed as commands
   - Other lines are parsed as expense entries
   - Bot name prefix and emojis are stripped
   - Supports both explicit dates (YYYY-MM-DD format) and implicit dates (message timestamp)
3. **Batching** (`batch.rs`): Commands are collected into batches per chat for atomic execution
4. **Execution** (`commands/mod.rs`): `execute_command()` dispatches to specific command handlers

### Command System

Commands are implemented using the `CommandTrait` pattern:

- Each command is a module in `commands/` (e.g., `command_help.rs`, `command_report.rs`)
- Commands implement `CommandTrait` which provides:
  - `parse_arguments()`: Custom parsing from command string
  - `run()`: Async execution with access to storage
  - `to_command_string()`: Serialization back to command format
- The main `Command` enum (in `commands/mod.rs`) aggregates all commands using `teloxide::BotCommands`

### Markdown Formatting (yoroolbot)

The `yoroolbot` library provides Telegram MarkdownV2 formatting utilities:

- **MarkdownString**: Type-safe builder for MarkdownV2 messages with proper escaping
- **validate_markdownv2_format**: Validates MarkdownV2 syntax
- **markdown_format!** macro: Convenient syntax for building messages

These utilities handle the complex escaping rules required by Telegram's MarkdownV2 format.

### Menu System

Interactive menus (`menus/` directory) use Telegram inline keyboards:

- `select_category.rs`: Category selection for filter operations
- `select_category_filter.rs`: Filter selection within categories
- `update_category.rs` / `update_category_filter.rs`: Edit/remove operations

Menus use type-safe callback data via the `CallbackData` enum in `handlers.rs`.

## Key Implementation Details

### Category Filtering
- Categories map to lists of regex patterns
- Expenses are matched against patterns case-insensitively
- Word extraction (`parser::extract_words()`) suggests filter words from uncategorized expenses
- Filter word selection supports pagination (20 words per page)

### Date Handling
- Expenses can have explicit dates (YYYY-MM-DD format) or use message timestamp
- All timestamps stored as Unix time (i64)
- Displayed using chrono formatting

### Cross-Compilation
- Target: `x86_64-unknown-linux-gnu`
- OpenSSL is statically compiled to avoid runtime dependencies
- Uses custom `.cargo/config.toml` for linker configuration

### Storage File Format
- Categories: YAML files in `categories/` directory (or custom path)
- Filename: `{chat_id}.yaml`
- Structure: `CategoryData` with `categories: HashMap<String, Vec<String>>`

## Testing Practices

The codebase has extensive unit tests, particularly for:
- Command parsing (`parser.rs`)
- Category data serialization (`storage.rs`)
- Expense handling

When adding features, maintain this testing coverage.
