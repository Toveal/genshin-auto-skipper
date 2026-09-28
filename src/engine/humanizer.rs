//! Randomized timing that mimics a human pressing keys.

use std::time::{Duration, Instant};

use rand::Rng;

use crate::settings::Settings;

pub struct Humanizer<R> {
    rng: R,
    last_break_check: Instant,
}

impl<R: Rng> Humanizer<R> {
    pub fn new(rng: R, now: Instant) -> Self {
        Self {
            rng,
            last_break_check: now,
        }
    }

    /// Delay before the next press.
    pub fn interval(&mut self, s: &Settings) -> Duration {
        if !s.humanize {
            return Duration::from_millis(s.interval_min_ms);
        }
        let ms = if self.rng.random_bool(s.slow_chance) {
            self.rng
                .random_range(s.interval_max_ms..=s.slow_interval_max_ms)
        } else {
            self.rng.random_range(s.interval_min_ms..=s.interval_max_ms)
        };
        Duration::from_millis(ms)
    }

    /// How long a key or button is held down.
    pub fn hold(&mut self, s: &Settings) -> Duration {
        let ms = if s.humanize {
            self.rng.random_range(30..=60)
        } else {
            40
        };
        Duration::from_millis(ms)
    }

    /// A random point in the inclusive rectangle.
    pub fn point_in(&mut self, min: (i32, i32), max: (i32, i32)) -> (i32, i32) {
        (
            self.rng.random_range(min.0..=max.0),
            self.rng.random_range(min.1..=max.1),
        )
    }

    /// Every `break_check_secs` rolls for a break; returns its length.
    pub fn maybe_break(&mut self, s: &Settings, now: Instant) -> Option<Duration> {
        if !s.breaks || now.duration_since(self.last_break_check).as_secs() < s.break_check_secs {
            return None;
        }
        self.last_break_check = now;
        self.rng.random_bool(s.break_chance).then(|| {
            Duration::from_secs_f64(self.rng.random_range(s.break_min_secs..=s.break_max_secs))
        })
    }

    pub fn reset_break_timer(&mut self, now: Instant) {
        self.last_break_check = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    fn humanizer(now: Instant) -> Humanizer<StdRng> {
        Humanizer::new(StdRng::seed_from_u64(7), now)
    }

    #[test]
    fn intervals_stay_in_reference_ranges() {
        let s = Settings::default();
        let mut h = humanizer(Instant::now());
        let (mut fast, mut slow) = (0, 0);
        for _ in 0..6_000 {
            let ms = h.interval(&s).as_millis() as u64;
            assert!((120..=200).contains(&ms), "{ms}");
            if ms > 180 {
                slow += 1;
            } else {
                fast += 1;
            }
        }
        // About one in six intervals is slow.
        assert!((700..1_300).contains(&slow), "{slow} slow / {fast} fast");
    }

    #[test]
    fn fixed_interval_without_humanize() {
        let s = Settings {
            humanize: false,
            ..Settings::default()
        };
        let mut h = humanizer(Instant::now());
        assert_eq!(h.interval(&s), Duration::from_millis(120));
        assert_eq!(h.hold(&s), Duration::from_millis(40));
    }

    #[test]
    fn breaks_only_after_check_period() {
        let s = Settings {
            break_chance: 1.0,
            ..Settings::default()
        };
        let start = Instant::now();
        let mut h = humanizer(start);
        assert_eq!(h.maybe_break(&s, start + Duration::from_secs(29)), None);
        let pause = h.maybe_break(&s, start + Duration::from_secs(30)).unwrap();
        assert!((3.0..=8.0).contains(&pause.as_secs_f64()));
        // The timer restarts after each check.
        assert_eq!(h.maybe_break(&s, start + Duration::from_secs(31)), None);
    }

    #[test]
    fn breaks_disabled() {
        let s = Settings {
            breaks: false,
            break_chance: 1.0,
            ..Settings::default()
        };
        let start = Instant::now();
        let mut h = humanizer(start);
        assert_eq!(h.maybe_break(&s, start + Duration::from_secs(600)), None);
    }

    #[test]
    fn point_in_rectangle() {
        let mut h = humanizer(Instant::now());
        for _ in 0..1_000 {
            let (x, y) = h.point_in((10, 20), (15, 22));
            assert!((10..=15).contains(&x) && (20..=22).contains(&y));
        }
    }
}
