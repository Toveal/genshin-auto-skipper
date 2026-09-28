//! Positions of the pixels the detector samples, for a given game resolution.
//!
//! Reference positions are measured at 1920×1080. For 16:9 they scale
//! proportionally; ultra-wide resolutions shift horizontally anchored UI
//! elements with the empirical formulas of genshin-dialogue-autoskip, and a few
//! resolutions have hand-measured overrides.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    /// 16:9, or any resolution not wider than 1920.
    Standard,
    /// Wider than 1920 and not 16:9.
    Wide,
    /// Hand-measured positions for this exact resolution.
    Measured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub width: i32,
    pub height: i32,
    pub profile: Profile,
    /// Autoplay button in the top-left corner, visible while a line plays.
    pub playing_icon: Point,
    /// Speech bubble of the bottom dialogue option (two possible heights).
    pub option_icon_lower: Point,
    pub option_icon_higher: Point,
    /// A point that is white while the loading screen is shown.
    pub loading_screen: Point,
    /// Inclusive rectangle inside the bottom dialogue option, for mouse clicks.
    pub option_zone_min: Point,
    pub option_zone_max: Point,
}

const BASE_W: i32 = 1920;
const BASE_H: i32 = 1080;

/// Hand-measured values that replace the computed ones.
struct Measured {
    resolution: (i32, i32),
    playing_icon_x: i32,
    option_icon_x: i32,
    option_icon_lower_y: i32,
    option_icon_higher_y: i32,
}

const MEASURED: &[Measured] = &[Measured {
    resolution: (2880, 1800),
    playing_icon_x: 126,
    option_icon_x: 1947,
    option_icon_lower_y: 1370,
    option_icon_higher_y: 1260,
}];

fn scale_x(x: i32, width: i32) -> i32 {
    x * width / BASE_W
}

fn scale_y(y: i32, height: i32) -> i32 {
    y * height / BASE_H
}

/// Position of an element bound to the left side: `at_1080p` at width 1920,
/// `at_3840` at width 3840, linear in between and beyond.
fn anchored_left(at_1080p: i32, at_3840: i32, width: i32) -> i32 {
    let per_pixel = f64::from(at_3840 - at_1080p) / f64::from(BASE_W);
    (f64::from(at_1080p) + f64::from(width - BASE_W) * per_pixel) as i32
}

/// Like [`anchored_left`] for right-bound elements; beyond 3840 the element
/// drifts by an additional `extra` per pixel.
fn anchored_right(at_1080p: i32, at_3840: i32, width: i32, extra: f64) -> i32 {
    let extra = if width <= 3840 { 0.0 } else { extra };
    let per_pixel = f64::from(at_3840 - at_1080p) / f64::from(BASE_W);
    (f64::from(at_1080p) + f64::from(width - BASE_W) * (per_pixel + extra)) as i32
}

fn is_standard_ratio(width: i32, height: i32) -> bool {
    width <= BASE_W || height * 16 == width * 9
}

impl Layout {
    pub fn new(width: i32, height: i32) -> Self {
        let (w, h) = (width, height);

        let measured = MEASURED.iter().find(|m| m.resolution == (w, h));

        let (profile, playing_x, option_x, lower_y, higher_y) = match measured {
            Some(m) => (
                Profile::Measured,
                m.playing_icon_x,
                m.option_icon_x,
                m.option_icon_lower_y,
                m.option_icon_higher_y,
            ),
            None if is_standard_ratio(w, h) => (
                Profile::Standard,
                scale_x(84, w),
                scale_x(1301, w),
                scale_y(808, h),
                scale_y(790, h),
            ),
            None => {
                let mut px = anchored_left(84, 230, w);
                if px > 231 {
                    px = 230;
                }
                (
                    Profile::Wide,
                    px,
                    anchored_right(1301, 2770, w, 0.02),
                    scale_y(810, h),
                    scale_y(792, h),
                )
            }
        };

        // The option text starts at the bubble and spans ~400 px at 1080p;
        // UI elements scale with the height.
        let mid_y = (lower_y + higher_y) / 2;
        let half_h = scale_y(5, h);

        Self {
            width,
            height,
            profile,
            playing_icon: Point {
                x: playing_x,
                y: scale_y(46, h),
            },
            option_icon_lower: Point {
                x: option_x,
                y: lower_y,
            },
            option_icon_higher: Point {
                x: option_x,
                y: higher_y,
            },
            loading_screen: Point {
                x: scale_x(1200, w),
                y: scale_y(700, h),
            },
            option_zone_min: Point {
                x: option_x + scale_y(20, h),
                y: mid_y - half_h,
            },
            option_zone_max: Point {
                x: option_x + scale_y(400, h),
                y: mid_y + half_h,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    #[test]
    fn full_hd_matches_reference() {
        let l = Layout::new(1920, 1080);
        assert_eq!(l.profile, Profile::Standard);
        assert_eq!(l.playing_icon, p(84, 46));
        assert_eq!(l.option_icon_lower, p(1301, 808));
        assert_eq!(l.option_icon_higher, p(1301, 790));
        assert_eq!(l.loading_screen, p(1200, 700));
    }

    #[test]
    fn qhd_scales_proportionally() {
        let l = Layout::new(2560, 1440);
        assert_eq!(l.profile, Profile::Standard);
        assert_eq!(l.playing_icon, p(112, 61));
        assert_eq!(l.option_icon_lower, p(1734, 1077));
        assert_eq!(l.option_icon_higher, p(1734, 1053));
        assert_eq!(l.loading_screen, p(1600, 933));
    }

    #[test]
    fn uhd_scales_proportionally() {
        let l = Layout::new(3840, 2160);
        assert_eq!(l.profile, Profile::Standard);
        assert_eq!(l.playing_icon, p(168, 92));
        assert_eq!(l.option_icon_lower, p(2602, 1616));
    }

    #[test]
    fn narrow_non_16_9_is_standard() {
        // Reference only switches profile for widths above 1920.
        assert_eq!(Layout::new(1680, 1050).profile, Profile::Standard);
    }

    #[test]
    fn ultrawide_2560x1080() {
        let l = Layout::new(2560, 1080);
        assert_eq!(l.profile, Profile::Wide);
        // 84 + 640 * 146/1920 = 132.67
        assert_eq!(l.playing_icon, p(132, 46));
        // 1301 + 640 * 1469/1920 = 1790.67
        assert_eq!(l.option_icon_lower, p(1790, 810));
        assert_eq!(l.option_icon_higher, p(1790, 792));
    }

    #[test]
    fn ultrawide_3440x1440() {
        let l = Layout::new(3440, 1440);
        assert_eq!(l.profile, Profile::Wide);
        // 84 + 1520 * 146/1920 = 199.58
        assert_eq!(l.playing_icon.x, 199);
        // 1301 + 1520 * 1469/1920 = 2463.96
        assert_eq!(l.option_icon_lower, p(2463, 1080));
        assert_eq!(l.option_icon_higher, p(2463, 1056));
    }

    #[test]
    fn super_ultrawide_5120x1440_uses_extra_drift_and_clamp() {
        let l = Layout::new(5120, 1440);
        assert_eq!(l.profile, Profile::Wide);
        // 84 + 3200 * 146/1920 = 327.3 -> clamped to 230
        assert_eq!(l.playing_icon.x, 230);
        // 1301 + 3200 * (1469/1920 + 0.02) = 3813.33
        assert_eq!(l.option_icon_lower.x, 3813);
    }

    #[test]
    fn measured_override_2880x1800() {
        let l = Layout::new(2880, 1800);
        assert_eq!(l.profile, Profile::Measured);
        assert_eq!(l.playing_icon, p(126, 76));
        assert_eq!(l.option_icon_lower, p(1947, 1370));
        assert_eq!(l.option_icon_higher, p(1947, 1260));
        assert_eq!(l.loading_screen, p(1800, 1166));
    }

    #[test]
    fn option_zone_is_inside_option_row() {
        for (w, h) in [(1920, 1080), (1280, 720), (2560, 1080), (2880, 1800)] {
            let l = Layout::new(w, h);
            assert!(l.option_zone_min.x < l.option_zone_max.x);
            assert!(l.option_zone_min.y <= l.option_zone_max.y);
            assert!(l.option_zone_min.x > l.option_icon_lower.x);
            assert!(l.option_zone_max.x < w);
        }
        let l = Layout::new(1920, 1080);
        assert_eq!(l.option_zone_min, p(1321, 794));
        assert_eq!(l.option_zone_max, p(1701, 804));
    }
}
