use crate::error::{AppError, Result};

pub fn get_text() -> Result<String> {
    arboard::Clipboard::new()
        .map_err(|e| AppError::Input(format!("Clipboard init failed: {e}")))?
        .get_text()
        .map_err(|e| AppError::Input(format!("Clipboard get failed: {e}")))
}

pub fn set_text(text: &str) -> Result<()> {
    arboard::Clipboard::new()
        .map_err(|e| AppError::Input(format!("Clipboard init failed: {e}")))?
        .set_text(text)
        .map_err(|e| AppError::Input(format!("Clipboard set failed: {e}")))
}
