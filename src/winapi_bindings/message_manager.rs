use super::bindings::{dispatch_message, get_global_message, translate_message};
use color_eyre::Result;
use winapi::um::winuser::LPMSG;

pub struct MessageManager {
    msg: LPMSG,
}

impl MessageManager {
    pub fn new() -> Self {
        MessageManager {
            msg: unsafe { std::mem::zeroed() },
        }
    }

    pub fn get_message(&mut self) -> Result<()> {
        get_global_message(self.msg)?;
        Ok(())
    }

    pub fn translate_message(&mut self) -> Result<()> {
        translate_message(self.msg)?;
        Ok(())
    }

    pub fn dispatch_message(&mut self) {
        dispatch_message(self.msg);
    }
}
