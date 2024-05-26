use once_cell::sync::OnceCell;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum EventType {
    KeyPress(u32),
    DestroyWindow,
    ChangeForegroundWindow,
    Shutdown,
}

pub static EVENT_LISTENER_CHANNEL: OnceCell<Sender<EventType>> = OnceCell::new();
