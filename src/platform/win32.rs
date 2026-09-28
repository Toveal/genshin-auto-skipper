//! Thin safe wrappers over the Win32 API. This module is the only place in the
//! crate that contains `unsafe` code for plain API calls.

use std::{io, mem, ptr};

use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, POINT, RECT},
    Globalization::GetUserDefaultUILanguage,
    Graphics::Gdi::{ClientToScreen, GetDC, GetPixel, ReleaseDC, CLR_INVALID, HDC},
    Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
    System::{
        Diagnostics::Debug::Beep,
        SystemInformation::GetLocalTime,
        Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken},
    },
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::{
            MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
            KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MOUSEEVENTF_LEFTDOWN,
            MOUSEEVENTF_LEFTUP, MOUSEINPUT,
        },
        WindowsAndMessaging::{
            FindWindowW, GetClientRect, GetCursorPos, GetForegroundWindow, IsIconic, IsWindow,
            MessageBoxW, SetCursorPos, MB_ICONERROR, MB_ICONWARNING, MB_OK,
        },
    },
};

/// Window handle that can be moved between threads. HWNDs are plain
/// identifiers, not pointers to memory owned by us.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hwnd(isize);

impl Hwnd {
    fn raw(self) -> HWND {
        self.0 as HWND
    }
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn last_error() -> io::Error {
    io::Error::last_os_error()
}

fn check(ok: i32) -> io::Result<()> {
    if ok != 0 {
        Ok(())
    } else {
        Err(last_error())
    }
}

// ---------------------------------------------------------------- windows ---

pub fn find_window(class: &str, title: &str) -> Option<Hwnd> {
    let (class, title) = (to_wide(class), to_wide(title));
    let hwnd = unsafe { FindWindowW(class.as_ptr(), title.as_ptr()) };
    (!hwnd.is_null()).then_some(Hwnd(hwnd as isize))
}

pub fn is_window(hwnd: Hwnd) -> bool {
    unsafe { IsWindow(hwnd.raw()) != 0 }
}

pub fn is_minimized(hwnd: Hwnd) -> bool {
    unsafe { IsIconic(hwnd.raw()) != 0 }
}

pub fn foreground_window() -> Option<Hwnd> {
    let hwnd = unsafe { GetForegroundWindow() };
    (!hwnd.is_null()).then_some(Hwnd(hwnd as isize))
}

/// Size of the client area (without borders and title bar).
pub fn client_size(hwnd: Hwnd) -> io::Result<(i32, i32)> {
    let mut rect: RECT = unsafe { mem::zeroed() };
    check(unsafe { GetClientRect(hwnd.raw(), &mut rect) })?;
    Ok((rect.right - rect.left, rect.bottom - rect.top))
}

/// Screen coordinates of the client-area origin.
pub fn client_origin(hwnd: Hwnd) -> io::Result<(i32, i32)> {
    let mut point = POINT { x: 0, y: 0 };
    check(unsafe { ClientToScreen(hwnd.raw(), &mut point) })?;
    Ok((point.x, point.y))
}

// ----------------------------------------------------------------- pixels ---

/// Device context of the whole (composited) screen. Reading the screen rather
/// than the game window works for DirectX swap chains, which do not render
/// into the window's GDI surface.
pub struct ScreenDc(HDC);

impl ScreenDc {
    pub fn new() -> io::Result<Self> {
        let hdc = unsafe { GetDC(ptr::null_mut()) };
        if hdc.is_null() {
            Err(last_error())
        } else {
            Ok(Self(hdc))
        }
    }

    pub fn pixel(&self, x: i32, y: i32) -> io::Result<(u8, u8, u8)> {
        let color = unsafe { GetPixel(self.0, x, y) };
        if color == CLR_INVALID {
            return Err(io::Error::other(format!("cannot read pixel at ({x}, {y})")));
        }
        Ok((color as u8, (color >> 8) as u8, (color >> 16) as u8))
    }
}

impl Drop for ScreenDc {
    fn drop(&mut self) {
        unsafe { ReleaseDC(ptr::null_mut(), self.0) };
    }
}

// ------------------------------------------------------------------ input ---

fn send_inputs(inputs: &[INPUT]) -> io::Result<()> {
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            mem::size_of::<INPUT>() as i32,
        )
    };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(last_error())
    }
}

/// Presses (`down == true`) or releases a key given by virtual-key code.
/// Scan codes are sent because games often ignore virtual-key-only input.
pub fn send_key(vk: u16, down: bool) -> io::Result<()> {
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    let mut flags = KEYEVENTF_SCANCODE;
    if !down {
        flags |= KEYEVENTF_KEYUP;
    }
    send_inputs(&[INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }])
}

pub fn send_left_button(down: bool) -> io::Result<()> {
    let flags = if down {
        MOUSEEVENTF_LEFTDOWN
    } else {
        MOUSEEVENTF_LEFTUP
    };
    send_inputs(&[INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }])
}

pub fn cursor_pos() -> io::Result<(i32, i32)> {
    let mut point = POINT { x: 0, y: 0 };
    check(unsafe { GetCursorPos(&mut point) })?;
    Ok((point.x, point.y))
}

pub fn set_cursor_pos(x: i32, y: i32) -> io::Result<()> {
    check(unsafe { SetCursorPos(x, y) })
}

// ---------------------------------------------------------------- process ---

pub fn enable_dpi_awareness() {
    // Fails harmlessly when the manifest has already set the awareness.
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}

/// Named mutex held for the lifetime of the process to prevent a second copy.
pub struct SingleInstance(HANDLE);

// SAFETY: the handle is only closed on drop and never used otherwise.
unsafe impl Send for SingleInstance {}

impl SingleInstance {
    /// Returns `None` when another instance already holds the mutex.
    pub fn acquire(name: &str) -> io::Result<Option<Self>> {
        let name = to_wide(name);
        let handle = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(last_error());
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return Ok(None);
        }
        Ok(Some(Self(handle)))
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

pub fn beep(frequency: u32, duration_ms: u32) {
    unsafe { Beep(frequency, duration_ms) };
}

/// Local wall-clock time as (hours, minutes, seconds).
pub fn local_time() -> (u16, u16, u16) {
    let mut t = unsafe { mem::zeroed() };
    unsafe { GetLocalTime(&mut t) };
    (t.wHour, t.wMinute, t.wSecond)
}

pub fn ui_language_is_russian() -> bool {
    const LANG_RUSSIAN: u16 = 0x19;
    (unsafe { GetUserDefaultUILanguage() } & 0x3ff) == LANG_RUSSIAN
}

pub fn message_box(title: &str, text: &str, error: bool) {
    let (title, text) = (to_wide(title), to_wide(text));
    let icon = if error { MB_ICONERROR } else { MB_ICONWARNING };
    unsafe { MessageBoxW(ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | icon) };
}
