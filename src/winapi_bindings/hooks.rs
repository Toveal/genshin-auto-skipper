use super::bindings::{
    set_win_event_hook, set_window_hook_keyboard_ll, unhook_win_event, unhook_windows_hook_ex,
};
use crate::{
    global_states::{EventType, EVENT_LISTENER_CHANNEL},
    winapi_bindings::bindings::set_console_ctrl_handler,
};
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use std::ptr::null_mut;
use winapi::{
    shared::{
        minwindef::{DWORD, LPARAM, LRESULT, UINT, WPARAM},
        windef::{HHOOK, HWINEVENTHOOK, HWND},
    },
    um::winuser::{
        CallNextHookEx, EVENT_OBJECT_DESTROY, EVENT_SYSTEM_FOREGROUND, HC_ACTION, KBDLLHOOKSTRUCT,
        WM_KEYDOWN,
    },
};

pub struct ChangeForegroundWindow {
    hook: HWINEVENTHOOK,
}

impl ChangeForegroundWindow {
    const HOOK_TYPE: UINT = EVENT_SYSTEM_FOREGROUND;

    pub fn new() -> Result<Self> {
        let hook = set_win_event_hook(Self::HOOK_TYPE, Some(Self::handler))?;
        Ok(Self { hook })
    }

    unsafe extern "system" fn handler(
        _: HWINEVENTHOOK,
        _: DWORD,
        _: HWND,
        _: winapi::um::winnt::LONG,
        _: winapi::um::winnt::LONG,
        _: DWORD,
        _: DWORD,
    ) {
        send_event(EventType::ChangeForegroundWindow).unwrap();
    }
}

impl Drop for ChangeForegroundWindow {
    fn drop(&mut self) {
        let _ = unhook_win_event(self.hook);
    }
}

pub struct DestroyWindow {
    hook: HWINEVENTHOOK,
}

impl DestroyWindow {
    const HOOK_TYPE: UINT = EVENT_OBJECT_DESTROY;

    pub fn new() -> Result<Self> {
        let hook = set_win_event_hook(Self::HOOK_TYPE, Some(Self::handler))?;
        Ok(Self { hook })
    }

    unsafe extern "system" fn handler(
        _: HWINEVENTHOOK,
        _: DWORD,
        _: HWND,
        _: winapi::um::winnt::LONG,
        _: winapi::um::winnt::LONG,
        _: DWORD,
        _: DWORD,
    ) {
        send_event(EventType::DestroyWindow).unwrap();
    }
}

impl Drop for DestroyWindow {
    fn drop(&mut self) {
        let _ = unhook_win_event(self.hook);
    }
}

pub struct ApplicationShutdown;

impl ApplicationShutdown {
    pub fn set() -> Result<()> {
        set_console_ctrl_handler(Some(Self::handler))?;
        Ok(())
    }

    unsafe extern "system" fn handler(_: u32) -> i32 {
        send_event(EventType::Shutdown).unwrap();
        1
    }
}

#[derive(Debug)]
pub struct KeyboardEvent {
    hook: HHOOK,
}

impl KeyboardEvent {
    pub fn new() -> Result<Self> {
        let hook = set_window_hook_keyboard_ll(Some(Self::handler))?;
        Ok(Self { hook })
    }

    unsafe extern "system" fn handler(
        code: std::ffi::c_int,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if code == HC_ACTION && w_param == WM_KEYDOWN as usize {
            let kbd_struct = &*(l_param as *const KBDLLHOOKSTRUCT);
            send_event(EventType::KeyPress(kbd_struct.vkCode)).unwrap();
        }

        CallNextHookEx(null_mut(), code, w_param, l_param)
    }
}

impl Drop for KeyboardEvent {
    fn drop(&mut self) {
        let _ = unhook_windows_hook_ex(self.hook);
    }
}

fn send_event(event: EventType) -> Result<()> {
    EVENT_LISTENER_CHANNEL
        .get()
        .wrap_err("event listener retrieval error")?
        .clone()
        .send(event)
        .wrap_err("event sending error")
}
