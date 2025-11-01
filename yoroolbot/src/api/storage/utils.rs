/// Encode a key to make it safe for use as a filename
///
/// This function encodes characters that are forbidden or problematic in filenames:
/// - `/` (path separator) -> `%2F`
/// - `\` (Windows path separator) -> `%5C`
/// - `:` (forbidden on Windows) -> `%3A`
/// - `*` (wildcard) -> `%2A`
/// - `?` (wildcard) -> `%3F`
/// - `"` (forbidden on Windows) -> `%22`
/// - `<` (forbidden on Windows) -> `%3C`
/// - `>` (forbidden on Windows) -> `%3E`
/// - `|` (forbidden on Windows) -> `%7C`
/// - `%` (our escape character) -> `%25`
/// - `.` at start (hidden files on Unix) -> `%2E`
/// - ` ` (space, can be problematic) -> `%20`
///
/// # Examples
/// ```
/// # use yoroolbot::storage::encode_key_to_filename;
/// assert_eq!(encode_key_to_filename("simple"), "simple");
/// assert_eq!(encode_key_to_filename("path/to/key"), "path%2Fto%2Fkey");
/// assert_eq!(encode_key_to_filename(".hidden"), "%2Ehidden");
/// assert_eq!(encode_key_to_filename("key:value"), "key%3Avalue");
/// ```
pub fn encode_key_to_filename(key: &str) -> String {
    let mut result = String::with_capacity(key.len());

    for (i, ch) in key.chars().enumerate() {
        match ch {
            '/' => result.push_str("%2F"),
            '\\' => result.push_str("%5C"),
            ':' => result.push_str("%3A"),
            '*' => result.push_str("%2A"),
            '?' => result.push_str("%3F"),
            '"' => result.push_str("%22"),
            '<' => result.push_str("%3C"),
            '>' => result.push_str("%3E"),
            '|' => result.push_str("%7C"),
            '%' => result.push_str("%25"),
            ' ' => result.push_str("%20"),
            '.' if i == 0 => result.push_str("%2E"), // Only encode leading dot
            _ => result.push(ch),
        }
    }

    result
}

/// Decode a filename back to the original key
///
/// This function reverses the encoding done by `encode_key_to_filename`.
///
/// # Examples
/// ```
/// # use yoroolbot::storage::{encode_key_to_filename, decode_filename_to_key};
/// assert_eq!(decode_filename_to_key("simple"), "simple");
/// assert_eq!(decode_filename_to_key("path%2Fto%2Fkey"), "path/to/key");
/// assert_eq!(decode_filename_to_key("%2Ehidden"), ".hidden");
/// assert_eq!(decode_filename_to_key("key%3Avalue"), "key:value");
///
/// // Round-trip test
/// let original = "path/to:key*.txt";
/// let encoded = encode_key_to_filename(original);
/// let decoded = decode_filename_to_key(&encoded);
/// assert_eq!(decoded, original);
/// ```
pub fn decode_filename_to_key(filename: &str) -> String {
    let mut result = String::with_capacity(filename.len());
    let mut chars = filename.chars();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Read next two characters as hex digits
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    // Invalid hex, keep the % and hex chars
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                // Not enough characters after %, keep the %
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

