pub mod extract_words;
pub mod parse_expenses;

/// Format Unix timestamp to a human-readable date string
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, TimeZone, Utc};
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d").to_string()
}
