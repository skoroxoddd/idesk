use crate::error::{AppError, Result};
use crate::input::events::{InputEvent, MouseButton, Modifiers};
use crate::input::injector::InputInjector;
use enigo::{
    Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings,
};

pub struct EnigoInjector {
    enigo: Enigo,
}

impl EnigoInjector {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Input(format!("Failed to create enigo: {e}")))?;
        Ok(Self { enigo })
    }

    fn map_key(key: &str) -> Option<Key> {
        match key.to_lowercase().as_str() {
            "enter" => Some(Key::Return),
            "escape" | "esc" => Some(Key::Escape),
            "backspace" => Some(Key::Backspace),
            "delete" => Some(Key::Delete),
            "tab" => Some(Key::Tab),
            "space" => Some(Key::Space),
            "arrowup" => Some(Key::UpArrow),
            "arrowdown" => Some(Key::DownArrow),
            "arrowleft" => Some(Key::LeftArrow),
            "arrowright" => Some(Key::RightArrow),
            "f1" => Some(Key::F1),
            "f2" => Some(Key::F2),
            "f3" => Some(Key::F3),
            "f4" => Some(Key::F4),
            "f5" => Some(Key::F5),
            "f6" => Some(Key::F6),
            "f7" => Some(Key::F7),
            "f8" => Some(Key::F8),
            "f9" => Some(Key::F9),
            "f10" => Some(Key::F10),
            "f11" => Some(Key::F11),
            "f12" => Some(Key::F12),
            _ if key.len() == 1 => None, // Regular char key, handled separately
            _ => None,
        }
    }

    fn apply_modifiers(&mut self, modifiers: &Modifiers) -> Result<()> {
        if modifiers.ctrl {
            self.enigo.key(Key::Control, Direction::Press)
                .map_err(|e| AppError::Input(format!("Failed to press Ctrl: {e}")))?;
        }
        if modifiers.alt {
            self.enigo.key(Key::Alt, Direction::Press)
                .map_err(|e| AppError::Input(format!("Failed to press Alt: {e}")))?;
        }
        if modifiers.shift {
            self.enigo.key(Key::Shift, Direction::Press)
                .map_err(|e| AppError::Input(format!("Failed to press Shift: {e}")))?;
        }
        if modifiers.meta {
            self.enigo.key(Key::Meta, Direction::Press)
                .map_err(|e| AppError::Input(format!("Failed to press Meta: {e}")))?;
        }
        Ok(())
    }

    fn release_modifiers(&mut self, modifiers: &Modifiers) -> Result<()> {
        if modifiers.meta {
            self.enigo.key(Key::Meta, Direction::Release)
                .map_err(|e| AppError::Input(format!("Failed to release Meta: {e}")))?;
        }
        if modifiers.shift {
            self.enigo.key(Key::Shift, Direction::Release)
                .map_err(|e| AppError::Input(format!("Failed to release Shift: {e}")))?;
        }
        if modifiers.alt {
            self.enigo.key(Key::Alt, Direction::Release)
                .map_err(|e| AppError::Input(format!("Failed to release Alt: {e}")))?;
        }
        if modifiers.ctrl {
            self.enigo.key(Key::Control, Direction::Release)
                .map_err(|e| AppError::Input(format!("Failed to release Ctrl: {e}")))?;
        }
        Ok(())
    }
}

impl InputInjector for EnigoInjector {
    fn inject(&mut self, event: &InputEvent) -> Result<()> {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.enigo.move(*x, *y)
                    .map_err(|e| AppError::Input(format!("Mouse move failed: {e}")))?;
            }
            InputEvent::MouseDown { button, x, y } => {
                self.enigo.move(*x, *y)
                    .map_err(|e| AppError::Input(format!("Mouse move failed: {e}")))?;
                let btn = map_mouse_button(button);
                self.enigo.button(btn, Direction::Press)
                    .map_err(|e| AppError::Input(format!("Mouse down failed: {e}")))?;
            }
            InputEvent::MouseUp { button, x, y } => {
                self.enigo.move(*x, *y)
                    .map_err(|e| AppError::Input(format!("Mouse move failed: {e}")))?;
                let btn = map_mouse_button(button);
                self.enigo.button(btn, Direction::Release)
                    .map_err(|e| AppError::Input(format!("Mouse up failed: {e}")))?;
            }
            InputEvent::MouseWheel { delta } => {
                self.enigo.scroll(-*delta)
                    .map_err(|e| AppError::Input(format!("Scroll failed: {e}")))?;
            }
            InputEvent::KeyDown { key, modifiers } => {
                self.apply_modifiers(modifiers)?;
                if let Some(special_key) = Self::map_key(key) {
                    self.enigo.key(special_key, Direction::Press)
                        .map_err(|e| AppError::Input(format!("Key down failed: {e}")))?;
                } else if key.len() == 1 {
                    let ch = key.chars().next().unwrap();
                    self.enigo.text(key)
                        .map_err(|e| AppError::Input(format!("Text input failed: {e}")))?;
                }
                self.release_modifiers(modifiers)?;
            }
            InputEvent::KeyUp { key, modifiers } => {
                if let Some(special_key) = Self::map_key(key) {
                    self.enigo.key(special_key, Direction::Release)
                        .map_err(|e| AppError::Input(format!("Key up failed: {e}")))?;
                }
            }
            InputEvent::ClipboardSet { text } => {
                self.clipboard_set(text)?;
            }
        }
        Ok(())
    }

    fn clipboard_get(&mut self) -> Result<String> {
        arboard::Clipboard::new()
            .map_err(|e| AppError::Input(format!("Clipboard init failed: {e}")))?
            .get_text()
            .map_err(|e| AppError::Input(format!("Clipboard get failed: {e}")))
    }

    fn clipboard_set(&mut self, text: &str) -> Result<()> {
        arboard::Clipboard::new()
            .map_err(|e| AppError::Input(format!("Clipboard init failed: {e}")))?
            .set_text(text)
            .map_err(|e| AppError::Input(format!("Clipboard set failed: {e}")))
    }
}

fn map_mouse_button(button: &MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Middle => Button::Middle,
        MouseButton::Right => Button::Right,
    }
}
