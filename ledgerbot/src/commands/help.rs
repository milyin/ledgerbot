use teloxide::types::{KeyboardButton, ReplyMarkup};

/// Create a persistent menu keyboard that shows on the left of the input field
pub fn create_menu_keyboard() -> ReplyMarkup {
    let keyboard = vec![vec![
        KeyboardButton::new("💡 /help"),
        KeyboardButton::new("🗒️ /list"),
        KeyboardButton::new("🗂 /categories"),
        KeyboardButton::new("📋 /report"),
    ]];
    ReplyMarkup::Keyboard(
        teloxide::types::KeyboardMarkup::new(keyboard)
            .resize_keyboard()
            .persistent(),
    )
}
