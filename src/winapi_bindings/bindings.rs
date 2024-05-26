use color_eyre::Result;
use std::ptr::null_mut;

use winapi::{
    shared::{
        minwindef::{FALSE, HINSTANCE, TRUE, WPARAM},
        windef::{HDC, HHOOK, HWINEVENTHOOK, HWND, RECT},
    },
    um::{
        consoleapi::SetConsoleCtrlHandler,
        wincon::PHANDLER_ROUTINE,
        wingdi::{GetBValue, GetGValue, GetPixel, GetRValue, CLR_INVALID},
        winuser::{
            DispatchMessageW, FindWindowW, GetDC, GetForegroundWindow, GetMessageW, GetWindowRect,
            ReleaseDC, SendMessageW, SetCursorPos, SetWinEventHook, SetWindowsHookExW,
            TranslateMessage, UnhookWinEvent, UnhookWindowsHookEx, HOOKPROC, LPMSG, VK_SPACE,
            WH_KEYBOARD_LL, WINEVENTPROC, WINEVENT_OUTOFCONTEXT, WM_KEYDOWN, WM_LBUTTONDOWN,
            WM_LBUTTONUP,
        },
    },
};

use super::utils::{last_os_error, make_lparam};

pub fn set_win_event_hook(event: u32, handler: WINEVENTPROC) -> Result<HWINEVENTHOOK> {
    match unsafe {
        SetWinEventHook(
            event,
            event,
            null_mut(),
            handler,
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    } {
        h if h.is_null() => Err(last_os_error()),
        h => Ok(h),
    }
}

pub fn set_window_hook_keyboard_ll(handler: HOOKPROC) -> Result<HHOOK> {
    match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, handler, null_mut() as HINSTANCE, 0) } {
        h if h.is_null() => Err(last_os_error()),
        h => Ok(h),
    }
}

pub fn get_global_message(msg: LPMSG) -> Result<()> {
    match unsafe { GetMessageW(msg, null_mut(), 0, 0) } {
        TRUE => Ok(()),
        FALSE => Err(last_os_error()),
        u => unreachable!(),
    }
}

pub fn translate_message(msg: LPMSG) -> Result<()> {
    match unsafe { TranslateMessage(msg) } {
        TRUE => Ok(()),
        FALSE => Err(last_os_error()),
        u => unreachable!(),
    }
}

pub fn dispatch_message(msg: LPMSG) {
    unsafe {
        DispatchMessageW(msg);
    }
}

pub fn unhook_win_event(hook: HWINEVENTHOOK) -> Result<()> {
    match unsafe { UnhookWinEvent(hook) } {
        TRUE => Ok(()),
        FALSE => Err(last_os_error()),
        u => unreachable!(),
    }
}

pub fn unhook_windows_hook_ex(hook: HHOOK) -> Result<()> {
    match unsafe { UnhookWindowsHookEx(hook) } {
        TRUE => Ok(()),
        FALSE => Err(last_os_error()),
        u => unreachable!(),
    }
}

pub fn set_console_ctrl_handler(hook: PHANDLER_ROUTINE) -> Result<()> {
    match unsafe { SetConsoleCtrlHandler(hook, 1) } {
        TRUE => Ok(()),
        FALSE => Err(last_os_error()),
        _ => unreachable!(),
    }
}

pub fn get_color_pixel(hwnd: HWND, x: i32, y: i32) -> Result<(u8, u8, u8)> {
    let hdc: HDC = unsafe { GetDC(hwnd) };

    if hdc.is_null() {
        return Err(last_os_error());
    }

    let color = unsafe { GetPixel(hdc, x, y) };
    if color == CLR_INVALID {
        unsafe { ReleaseDC(hwnd, hdc) };
        return Err(last_os_error());
    }

    match unsafe { ReleaseDC(hwnd, hdc) } {
        0 => return Err(last_os_error()),
        1 => {}
        u => unreachable!(),
    };
    let rgb = (GetRValue(color), GetGValue(color), GetBValue(color));
    Ok(rgb)
}

pub fn set_cursor_position(x: i32, y: i32) -> Result<()> {
    match unsafe { SetCursorPos(x, y) } {
        TRUE => Ok(()),
        FALSE => Err(last_os_error()),
        _ => unreachable!(),
    }
}

pub fn get_foreground_window() -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

pub fn get_window_size(hwnd: HWND) -> Result<(i32, i32)> {
    let mut rect: RECT = unsafe { std::mem::zeroed() };
    if unsafe { GetWindowRect(hwnd, &mut rect) } != 0 {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        Ok((width, height))
    } else {
        Err(last_os_error())
    }
}

pub fn send_message_space(hwnd: HWND, key_press: bool) -> Result<()> {
    let flags = if key_press { 0x8000 } else { 0x0000 };
    let l_param = make_lparam(1, flags | VK_SPACE);
    let result = unsafe { SendMessageW(hwnd, WM_KEYDOWN, VK_SPACE as WPARAM, l_param) };
    if result == 0 {
        Ok(())
    } else {
        Err(last_os_error())
    }
}

pub fn send_message_click(hwnd: HWND, key_press: bool, pos_x: i32, pos_y: i32) -> Result<()> {
    let l_param = make_lparam(pos_x, pos_y);
    let msg = if key_press {
        WM_LBUTTONDOWN
    } else {
        WM_LBUTTONUP
    };
    let result = unsafe { SendMessageW(hwnd, msg, 0 as WPARAM, l_param) };
    if result == 0 {
        Ok(())
    } else {
        Err(last_os_error())
    }
}

pub fn find_window(w_name: &[u16], w_class_name: &[u16]) -> Result<HWND> {
    let hwnd = unsafe { FindWindowW(w_class_name.as_ptr(), w_name.as_ptr()) };
    if hwnd.is_null() {
        Err(last_os_error())
    } else {
        Ok(hwnd)
    }
}
