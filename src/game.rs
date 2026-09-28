//! The Genshin Impact window and the input sent to it.

use std::{io, thread, time::Duration};

use crate::{
    engine::{
        detector::{PixelSource, Rgb},
        layout::Point,
    },
    platform::win32::{self, Hwnd, ScreenDc},
};

const CLASS_NAME: &str = "UnityWndClass";
/// Global and Chinese client titles.
const TITLES: [&str; 2] = ["Genshin Impact", "原神"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameWindow(Hwnd);

impl GameWindow {
    pub fn find() -> Option<Self> {
        TITLES
            .iter()
            .find_map(|title| win32::find_window(CLASS_NAME, title))
            .map(Self)
    }

    pub fn exists(&self) -> bool {
        win32::is_window(self.0)
    }

    pub fn is_foreground(&self) -> bool {
        win32::foreground_window() == Some(self.0)
    }

    pub fn is_minimized(&self) -> bool {
        win32::is_minimized(self.0)
    }

    pub fn client_size(&self) -> io::Result<(i32, i32)> {
        win32::client_size(self.0)
    }

    /// Starts sampling the game's client area from the screen.
    pub fn capture(&self) -> io::Result<Capture> {
        Ok(Capture {
            dc: ScreenDc::new()?,
            origin: win32::client_origin(self.0)?,
        })
    }

    pub fn client_to_screen(&self, at: Point) -> io::Result<(i32, i32)> {
        let (ox, oy) = win32::client_origin(self.0)?;
        Ok((ox + at.x, oy + at.y))
    }
}

pub struct Capture {
    dc: ScreenDc,
    origin: (i32, i32),
}

impl PixelSource for Capture {
    fn pixel(&self, at: Point) -> io::Result<Rgb> {
        self.dc.pixel(self.origin.0 + at.x, self.origin.1 + at.y)
    }
}

pub fn tap_key(vk: u16, hold: Duration) -> io::Result<()> {
    win32::send_key(vk, true)?;
    thread::sleep(hold);
    win32::send_key(vk, false)
}

/// Clicks at a screen position and puts the cursor back where it was.
pub fn click_at(x: i32, y: i32, hold: Duration) -> io::Result<()> {
    let previous = win32::cursor_pos().ok();
    win32::set_cursor_pos(x, y)?;
    win32::send_left_button(true)?;
    thread::sleep(hold);
    let result = win32::send_left_button(false);
    if let Some((px, py)) = previous {
        let _ = win32::set_cursor_pos(px, py);
    }
    result
}
