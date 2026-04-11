use crate::error::Result;
use crate::input::events::InputEvent;

pub trait InputInjector: Send + Sync {
    fn inject(&mut self, event: &InputEvent) -> Result<()>;
    fn clipboard_get(&mut self) -> Result<String>;
    fn clipboard_set(&mut self, text: &str) -> Result<()>;
}
