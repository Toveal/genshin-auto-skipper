//! Decides what is on screen from a handful of sampled pixels.

use std::io;

use super::layout::{Layout, Point};

pub type Rgb = (u8, u8, u8);

/// Color of the autoplay button while a dialogue line is playing.
pub const PLAYING_ICON_COLOR: Rgb = (236, 229, 216);
pub const WHITE: Rgb = (255, 255, 255);

/// Source of pixels in game-client coordinates.
pub trait PixelSource {
    fn pixel(&self, at: Point) -> io::Result<Rgb>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogState {
    /// No dialogue on screen.
    None,
    Loading,
    /// A line is playing; the interaction key advances it.
    Talking,
    /// Dialogue options are shown.
    Choice,
}

pub fn color_matches(actual: Rgb, expected: Rgb, tolerance: u8) -> bool {
    actual.0.abs_diff(expected.0) <= tolerance
        && actual.1.abs_diff(expected.1) <= tolerance
        && actual.2.abs_diff(expected.2) <= tolerance
}

pub fn detect(src: &impl PixelSource, layout: &Layout, tolerance: u8) -> io::Result<DialogState> {
    let is = |at: Point, color: Rgb| -> io::Result<bool> {
        Ok(color_matches(src.pixel(at)?, color, tolerance))
    };

    if is(layout.loading_screen, WHITE)? {
        // The option bubble is white on a loading screen too, so it cannot be
        // trusted; the autoplay button can.
        return Ok(if is(layout.playing_icon, PLAYING_ICON_COLOR)? {
            DialogState::Talking
        } else {
            DialogState::Loading
        });
    }

    if is(layout.option_icon_lower, WHITE)? || is(layout.option_icon_higher, WHITE)? {
        return Ok(DialogState::Choice);
    }

    if is(layout.playing_icon, PLAYING_ICON_COLOR)? {
        return Ok(DialogState::Talking);
    }

    Ok(DialogState::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const DARK: Rgb = (30, 30, 40);

    struct Fake(HashMap<(i32, i32), Rgb>);

    impl Fake {
        fn new(layout: &Layout, pixels: &[(Point, Rgb)]) -> Self {
            let mut map = HashMap::new();
            for pt in [
                layout.playing_icon,
                layout.option_icon_lower,
                layout.option_icon_higher,
                layout.loading_screen,
            ] {
                map.insert((pt.x, pt.y), DARK);
            }
            for (pt, c) in pixels {
                map.insert((pt.x, pt.y), *c);
            }
            Self(map)
        }
    }

    impl PixelSource for Fake {
        fn pixel(&self, at: Point) -> io::Result<Rgb> {
            self.0
                .get(&(at.x, at.y))
                .copied()
                .ok_or_else(|| io::Error::other(format!("unexpected sample {at:?}")))
        }
    }

    fn run(pixels: &[(Point, Rgb)], tolerance: u8) -> DialogState {
        let layout = Layout::new(1920, 1080);
        let fake = Fake::new(&layout, pixels);
        detect(&fake, &layout, tolerance).unwrap()
    }

    #[test]
    fn nothing_on_screen() {
        assert_eq!(run(&[], 0), DialogState::None);
    }

    #[test]
    fn talking() {
        let l = Layout::new(1920, 1080);
        assert_eq!(
            run(&[(l.playing_icon, PLAYING_ICON_COLOR)], 0),
            DialogState::Talking
        );
    }

    #[test]
    fn choice_lower_and_higher() {
        let l = Layout::new(1920, 1080);
        assert_eq!(run(&[(l.option_icon_lower, WHITE)], 0), DialogState::Choice);
        assert_eq!(
            run(&[(l.option_icon_higher, WHITE)], 0),
            DialogState::Choice
        );
    }

    #[test]
    fn choice_wins_over_playing_icon() {
        let l = Layout::new(1920, 1080);
        let state = run(
            &[
                (l.playing_icon, PLAYING_ICON_COLOR),
                (l.option_icon_lower, WHITE),
            ],
            0,
        );
        assert_eq!(state, DialogState::Choice);
    }

    #[test]
    fn loading_screen_suppresses_option_bubble() {
        let l = Layout::new(1920, 1080);
        let state = run(
            &[(l.loading_screen, WHITE), (l.option_icon_lower, WHITE)],
            0,
        );
        assert_eq!(state, DialogState::Loading);
    }

    #[test]
    fn tolerance() {
        let l = Layout::new(1920, 1080);
        let near = (230, 233, 210);
        assert_eq!(run(&[(l.playing_icon, near)], 0), DialogState::None);
        assert_eq!(run(&[(l.playing_icon, near)], 6), DialogState::Talking);
        assert!(color_matches((250, 250, 250), WHITE, 5));
        assert!(!color_matches((249, 255, 255), WHITE, 5));
    }
}
