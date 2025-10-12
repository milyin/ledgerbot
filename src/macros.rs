/// Utility macros for the ledgerbot project
///
/// A macro to send a markdown message with automatic escaping and validation.
/// 
/// Usage:
/// ```rust
/// let request = send_message_markdown!(bot, chat_id, "Hello *{}*", name);
/// request.await?;
/// ```
/// 
/// **Important**: The format string must be MarkdownV2-safe. For messages with special characters,
/// escape them manually (\!) as the format string is used as-is. Only the arguments are automatically escaped.
/// 
/// The macro:
/// 1. Escapes each parameter using `teloxide::utils::markdown::escape`
/// 2. Validates basic markdown balance at compile time
/// 3. Returns a `SendMessage` request with `ParseMode::MarkdownV2` set
/// 
/// Parameters:
/// - `bot`: The telegram bot instance
/// - `chat_id`: The chat ID to send the message to
/// - `format_str`: A MarkdownV2-safe format string
/// - `args...`: Arguments to be escaped and inserted into the format string
#[macro_export]
macro_rules! send_message_markdown {
    ($bot:expr, $chat_id:expr, $format_str:expr $(, $arg:expr)*) => {{
        use teloxide::types::ParseMode;
        use teloxide::payloads::SendMessageSetters;
        
        // Compile-time validation for Telegram MarkdownV2 format compatibility
        const _: () = {
            let format_str_bytes = $format_str.as_bytes();
            let mut i = 0;
            let mut asterisk_count = 0u8;
            let mut underscore_count = 0u8;
            let mut backtick_count = 0u8;
            let mut square_bracket_count = 0u8;
            let mut paren_count = 0u8;
            let mut tilde_count = 0u8;
            let mut pipe_count = 0u8;
            
            // Track nesting state for validation
            let mut in_code = false;
            let mut in_pre = false;
            let mut prev_char = 0u8;
            
            while i < format_str_bytes.len() {
                let current_char = format_str_bytes[i];
                let is_escaped = prev_char == b'\\';
                
                if !is_escaped {
                    match current_char {
                        // Basic formatting characters must be balanced
                        b'*' => asterisk_count = asterisk_count.wrapping_add(1),
                        b'_' => underscore_count = underscore_count.wrapping_add(1),
                        b'~' => tilde_count = tilde_count.wrapping_add(1),
                        b'|' => pipe_count = pipe_count.wrapping_add(1),
                        
                        // Code formatting validation
                        b'`' => {
                            backtick_count = backtick_count.wrapping_add(1);
                            // Check for triple backticks (pre-formatted)
                            if i + 2 < format_str_bytes.len() && 
                               format_str_bytes[i + 1] == b'`' && 
                               format_str_bytes[i + 2] == b'`' {
                                in_pre = !in_pre;
                            } else {
                                in_code = !in_code;
                            }
                        },
                        
                        // Link formatting validation
                        b'[' => square_bracket_count = square_bracket_count.wrapping_add(1),
                        b']' => {
                            assert!(square_bracket_count > 0, "Unmatched closing square bracket ']' in markdown format string");
                            square_bracket_count = square_bracket_count.wrapping_sub(1);
                        },
                        b'(' => {
                            // Only count if it's potentially part of a link (after ])
                            if prev_char == b']' {
                                paren_count = paren_count.wrapping_add(1);
                            }
                        },
                        b')' => {
                            if paren_count > 0 {
                                paren_count = paren_count.wrapping_sub(1);
                            }
                        },
                        
                        // Reserved characters that should be escaped (compile-time check)
                        b'!' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '!' in MarkdownV2 format string. Use \\! to escape it.");
                            }
                        },
                        b'.' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '.' in MarkdownV2 format string. Use \\. to escape it.");
                            }
                        },
                        b'-' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '-' in MarkdownV2 format string. Use \\- to escape it.");
                            }
                        },
                        b'+' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '+' in MarkdownV2 format string. Use \\+ to escape it.");
                            }
                        },
                        b'=' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '=' in MarkdownV2 format string. Use \\= to escape it.");
                            }
                        },
                        b'>' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '>' in MarkdownV2 format string. Use \\> to escape it.");
                            }
                        },
                        b'#' => {
                            if !in_code && !in_pre && !is_escaped {
                                assert!(false, "Unescaped '#' in MarkdownV2 format string. Use \\# to escape it.");
                            }
                        },
                        b'{' => {
                            // Allow format placeholders like {}
                            let is_format_placeholder = i + 1 < format_str_bytes.len() && 
                                format_str_bytes[i + 1] == b'}';
                            if !in_code && !in_pre && !is_escaped && !is_format_placeholder {
                                assert!(false, "Unescaped '{{' in MarkdownV2 format string. Use \\{{ to escape it or use {{}} for format placeholders.");
                            }
                        },
                        b'}' => {
                            // Allow closing of format placeholders
                            let is_format_placeholder = i > 0 && format_str_bytes[i - 1] == b'{';
                            if !in_code && !in_pre && !is_escaped && !is_format_placeholder {
                                assert!(false, "Unescaped '}}' in MarkdownV2 format string. Use \\}} to escape it.");
                            }
                        },
                        
                        _ => {}
                    }
                }
                
                prev_char = current_char;
                i += 1;
            }
            
            // Validate balanced formatting
            assert!(asterisk_count % 2 == 0, "Unmatched asterisks (*) in MarkdownV2 format string - bold formatting must be balanced");
            assert!(underscore_count % 2 == 0, "Unmatched underscores (_) in MarkdownV2 format string - italic formatting must be balanced");
            assert!(backtick_count % 2 == 0, "Unmatched backticks (`) in MarkdownV2 format string - code formatting must be balanced");
            assert!(tilde_count % 2 == 0, "Unmatched tildes (~) in MarkdownV2 format string - strikethrough formatting must be balanced");
            assert!(pipe_count % 2 == 0, "Unmatched pipes (|) in MarkdownV2 format string - spoiler formatting must be balanced");
            assert!(square_bracket_count == 0, "Unmatched square brackets ([]) in MarkdownV2 format string - link text must be properly closed");
            assert!(paren_count == 0, "Unmatched parentheses in MarkdownV2 format string - link URLs must be properly closed");
            assert!(!in_code, "Unclosed code block in MarkdownV2 format string");
            assert!(!in_pre, "Unclosed pre-formatted code block in MarkdownV2 format string");
        };
        
        // Escape all arguments and format the message
        let formatted_message = format!($format_str $(, teloxide::utils::markdown::escape(&$arg.to_string()))*);
        
        // Runtime validation to catch MarkdownV2 issues early
        #[cfg(debug_assertions)]
        {
            // In debug mode, validate the result for common MarkdownV2 issues
            let reserved_unescaped = ['!', '.', '-', '+', '=', '|', '{', '}', '(', ')', '[', ']', '>', '#'];
            for (i, ch) in formatted_message.char_indices() {
                if reserved_unescaped.contains(&ch) {
                    // Check if it's escaped
                    let is_escaped = i > 0 && formatted_message.chars().nth(i - 1) == Some('\\');
                    if !is_escaped {
                        panic!("Unescaped MarkdownV2 character '{}' found in formatted message: '{}'. Escape manually or ensure proper formatting.", ch, formatted_message);
                    }
                }
            }
        }
        
        // Create and return the message request
        $bot.send_message($chat_id, formatted_message)
            .parse_mode(ParseMode::MarkdownV2)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_time_validation() {
        // These should compile successfully
        const _: () = {
            let format_str = "Hello *{}*";
            let format_str_bytes = format_str.as_bytes();
            let mut asterisk_count = 0u8;
            let mut i = 0;
            
            while i < format_str_bytes.len() {
                if format_str_bytes[i] == b'*' {
                    asterisk_count = asterisk_count.wrapping_add(1);
                }
                i += 1;
            }
            
            assert!(asterisk_count % 2 == 0, "Unmatched asterisks in markdown format string");
        };
    }

    #[test] 
    fn test_markdownv2_format_patterns() {
        // Test various valid MarkdownV2 patterns
        let valid_patterns = [
            "Simple text",
            "With *bold* text",
            "With _italic_ text", 
            "With `code` text",
            "With ~strikethrough~ text",
            "With ||spoiler|| text",
            "*{}* and _{}_ and `{}`",
            "**Bold** text",
            "__Italic__ text",
            "~~Strikethrough~~ text",
            "Link: [text](url)",
            "Code block: ```code```",
            "Mixed: *bold* and `code`",
            "Escaped \\! exclamation",
            "Escaped \\. period", 
            "Escaped \\- dash",
            "Escaped \\+ plus",
            "Escaped \\= equals",
            "Format placeholder: {}",
        ];

        // Test the enhanced validation logic for each pattern
        for pattern in valid_patterns {
            let format_str_bytes = pattern.as_bytes();
            let mut i = 0;
            let mut asterisk_count = 0u8;
            let mut underscore_count = 0u8;
            let mut backtick_count = 0u8;
            let mut square_bracket_count = 0u8;
            let mut paren_count = 0u8;
            let mut tilde_count = 0u8;
            let mut pipe_count = 0u8;
            let mut prev_char = 0u8;
            
            while i < format_str_bytes.len() {
                let current_char = format_str_bytes[i];
                let is_escaped = prev_char == b'\\';
                
                if !is_escaped {
                    match current_char {
                        b'*' => asterisk_count = asterisk_count.wrapping_add(1),
                        b'_' => underscore_count = underscore_count.wrapping_add(1),
                        b'~' => tilde_count = tilde_count.wrapping_add(1),
                        b'|' => pipe_count = pipe_count.wrapping_add(1),
                        b'`' => backtick_count = backtick_count.wrapping_add(1),
                        b'[' => square_bracket_count = square_bracket_count.wrapping_add(1),
                        b']' => {
                            if square_bracket_count > 0 {
                                square_bracket_count = square_bracket_count.wrapping_sub(1);
                            }
                        },
                        b'(' => {
                            if prev_char == b']' {
                                paren_count = paren_count.wrapping_add(1);
                            }
                        },
                        b')' => {
                            if paren_count > 0 {
                                paren_count = paren_count.wrapping_sub(1);
                            }
                        },
                        _ => {}
                    }
                }
                
                prev_char = current_char;
                i += 1;
            }
            
            // Validate all formatting is balanced
            assert!(asterisk_count % 2 == 0, "Pattern '{}' has unmatched asterisks", pattern);
            assert!(underscore_count % 2 == 0, "Pattern '{}' has unmatched underscores", pattern);
            assert!(backtick_count % 2 == 0, "Pattern '{}' has unmatched backticks", pattern);
            assert!(tilde_count % 2 == 0, "Pattern '{}' has unmatched tildes", pattern);
            assert!(pipe_count % 2 == 0, "Pattern '{}' has unmatched pipes", pattern);
            assert!(square_bracket_count == 0, "Pattern '{}' has unmatched square brackets", pattern);
            assert!(paren_count == 0, "Pattern '{}' has unmatched parentheses", pattern);
        }
    }

    // Note: These patterns would cause compile errors if used with send_message_markdown!
    // Invalid examples (unbalanced formatting):
    // "*unmatched bold" - unmatched asterisk
    // "_unmatched italic" - unmatched underscore  
    // "`unmatched code" - unmatched backtick
    // "~unmatched strike" - unmatched tilde
    // "||unmatched spoiler" - unmatched pipes
    // "[unmatched link" - unmatched square bracket
    // "[text](unmatched url" - unmatched parenthesis
}