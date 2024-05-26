use crate::winapi_bindings::{
    bindings::{
        find_window, get_color_pixel, get_foreground_window, get_window_size, send_message_click,
        send_message_space, set_cursor_position,
    },
    utils::to_wide_string,
};
use color_eyre::Result;
use once_cell::sync::OnceCell;
use rand::Rng;
use std::{thread::sleep, time::Duration};
use winapi::shared::windef::HWND;

static GENSHIN_WINDOW_NAME: OnceCell<Vec<u16>> = OnceCell::new();
static GENSHIN_WINDOW_CLASS_NAME: OnceCell<Vec<u16>> = OnceCell::new();

#[derive(Clone, Copy, Debug)]
pub struct WindowProps {
    bottom_dialogue_min_x: i32,
    bottom_dialogue_max_x: i32,
    bottom_dialogue_min_y: i32,
    bottom_dialogue_max_y: i32,
    playing_icon_x: i32,
    playing_icon_y: i32,
    dialogue_icon_x: i32,
    dialogue_icon_lower_y: i32,
    dialogue_icon_higher_y: i32,
    loading_screen_x: i32,
    loading_screen_y: i32,
}

impl WindowProps {
    const DEFAULT_WIDTH: i32 = 1920;
    const DEFAULT_HEIGHT: i32 = 1080;

    pub fn new(window: &Window) -> Result<Self> {
        let (w_width, w_height) = get_window_size(window.hwnd())?;

        Ok(Self {
            bottom_dialogue_min_x: Self::width_adjust(1300, w_width),
            bottom_dialogue_max_x: Self::width_adjust(1700, w_width),
            bottom_dialogue_min_y: Self::height_adjust(790, w_height),
            bottom_dialogue_max_y: Self::height_adjust(800, w_height),
            playing_icon_x: Self::width_adjust(84, w_width),
            playing_icon_y: Self::height_adjust(46, w_height),
            dialogue_icon_x: Self::width_adjust(1301, w_width),
            dialogue_icon_lower_y: Self::height_adjust(808, w_height),
            dialogue_icon_higher_y: Self::height_adjust(790, w_height),
            loading_screen_x: Self::width_adjust(1200, w_width),
            loading_screen_y: Self::height_adjust(700, w_height),
        })
    }

    fn width_adjust(width: i32, window_width: i32) -> i32 {
        width * window_width / Self::DEFAULT_WIDTH
    }

    fn height_adjust(height: i32, window_height: i32) -> i32 {
        height * window_height / Self::DEFAULT_HEIGHT
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Window {
    hwnd: usize,
}

impl Window {
    pub const DEFAULT_DURATION: Duration = Duration::from_millis(50);

    pub fn new() -> Result<Self> {
        let window_name = GENSHIN_WINDOW_NAME.get_or_init(|| to_wide_string("Genshin Impact"));
        let class_name = GENSHIN_WINDOW_CLASS_NAME.get_or_init(|| to_wide_string("UnityWndClass"));

        let hwnd = find_window(window_name, class_name)?;
        Ok(Self {
            hwnd: hwnd as usize,
        })
    }

    pub fn dialog_played(&self, props: &WindowProps) -> Result<bool> {
        let hwnd = self.hwnd();

        let dialog_icon_color = get_color_pixel(hwnd, props.playing_icon_x, props.playing_icon_y)?;
        if dialog_icon_color == (236, 229, 216) {
            return Ok(true);
        }

        let white_pixel = (255, 255, 255);
        if get_color_pixel(hwnd, props.loading_screen_x, props.loading_screen_y)? == white_pixel {
            return Ok(false);
        }

        if get_color_pixel(hwnd, props.dialogue_icon_x, props.dialogue_icon_lower_y)? == white_pixel
        {
            return Ok(true);
        }

        if get_color_pixel(hwnd, props.dialogue_icon_x, props.dialogue_icon_higher_y)?
            == white_pixel
        {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn is_dialog_without_option(&self, props: &WindowProps) -> Result<bool> {
        let dialog_icon_color =
            get_color_pixel(self.hwnd(), props.playing_icon_x, props.playing_icon_y)?;
        if dialog_icon_color == (236, 229, 216) {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn click_space(&self) -> Result<()> {
        let hwnd = self.hwnd();
        send_message_space(hwnd, true)?;
        sleep(Self::DEFAULT_DURATION);
        send_message_space(hwnd, false)?;
        Ok(())
    }

    pub fn click_left_m_button_random_pos(&self, props: &WindowProps) -> Result<()> {
        let hwnd = self.hwnd();
        let mut rnd = rand::thread_rng();
        let pos_x = rnd.gen_range(props.bottom_dialogue_min_x..=props.bottom_dialogue_max_x);
        let pos_y = rnd.gen_range(props.bottom_dialogue_min_y..=props.bottom_dialogue_max_y);
        set_cursor_position(pos_x, pos_y)?;
        send_message_click(hwnd, true, pos_x, pos_y)?;
        sleep(Self::DEFAULT_DURATION);
        send_message_click(hwnd, false, pos_x, pos_y)?;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        if let Some(fg_w) = get_foreground_window() {
            self.hwnd() == fg_w
        } else {
            false
        }
    }

    fn hwnd(&self) -> HWND {
        self.hwnd as HWND
    }
}
