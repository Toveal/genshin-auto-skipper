//! User settings, persisted as TOML in `%APPDATA%\genshin-auto-skipper`.
//! Defaults reproduce the behavior of genshin-dialogue-autoskip.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::platform::keys::{self, VK_F, VK_F10, VK_F8, VK_F9};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Ru,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkipMode {
    /// Act only when a dialogue is detected on screen.
    Detect,
    /// Press the interaction key at the humanized interval, without
    /// looking at the screen.
    Press,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionMode {
    /// Interaction key for lines, confirm key for options.
    Keyboard,
    /// Interaction key for lines, mouse click on the bottom option.
    Mouse,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: Language,

    pub skip_mode: SkipMode,
    pub action_mode: ActionMode,
    pub interact_key: u16,
    pub confirm_key: u16,

    pub hotkey_start: Option<u16>,
    pub hotkey_pause: Option<u16>,
    pub hotkey_exit: Option<u16>,

    /// Randomize timings. When off, presses repeat every `interval_min_ms`.
    pub humanize: bool,
    pub interval_min_ms: u64,
    pub interval_max_ms: u64,
    /// With `slow_chance`, the interval is drawn from
    /// `interval_max_ms..slow_interval_max_ms` instead.
    pub slow_interval_max_ms: u64,
    pub slow_chance: f64,

    pub breaks: bool,
    pub break_check_secs: u64,
    pub break_chance: f64,
    pub break_min_secs: f64,
    pub break_max_secs: f64,

    /// Maximum per-channel difference when matching pixel colors.
    pub color_tolerance: u8,
    pub sound: bool,
    pub always_on_top: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: if crate::platform::win32::ui_language_is_russian() {
                Language::Ru
            } else {
                Language::En
            },
            skip_mode: SkipMode::Detect,
            action_mode: ActionMode::Keyboard,
            interact_key: VK_F,
            confirm_key: VK_F,
            hotkey_start: Some(VK_F8),
            hotkey_pause: Some(VK_F9),
            // The reference uses F12, which Windows reserves for debuggers.
            hotkey_exit: Some(VK_F10),
            humanize: true,
            interval_min_ms: 120,
            interval_max_ms: 180,
            slow_interval_max_ms: 200,
            slow_chance: 1.0 / 6.0,
            breaks: true,
            break_check_secs: 30,
            break_chance: 1.0 / 25.0,
            break_min_secs: 3.0,
            break_max_secs: 8.0,
            color_tolerance: 4,
            sound: true,
            always_on_top: false,
        }
    }
}

impl Settings {
    /// Clamps values into sane ranges so hand-edited files cannot break timing.
    pub fn sanitize(&mut self) {
        self.interval_min_ms = self.interval_min_ms.clamp(30, 5_000);
        self.interval_max_ms = self.interval_max_ms.clamp(self.interval_min_ms, 5_000);
        self.slow_interval_max_ms = self
            .slow_interval_max_ms
            .clamp(self.interval_max_ms, 10_000);
        self.slow_chance = clamp_probability(self.slow_chance);
        self.break_check_secs = self.break_check_secs.clamp(1, 3_600);
        self.break_chance = clamp_probability(self.break_chance);
        self.break_min_secs = finite_or(self.break_min_secs, 3.0).clamp(0.5, 600.0);
        self.break_max_secs = finite_or(self.break_max_secs, 8.0).clamp(self.break_min_secs, 600.0);
        self.color_tolerance = self.color_tolerance.min(64);
        for hotkey in [
            &mut self.hotkey_start,
            &mut self.hotkey_pause,
            &mut self.hotkey_exit,
        ] {
            *hotkey = hotkey.filter(|&vk| keys::can_be_hotkey(vk));
        }
    }

    pub fn dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("genshin-auto-skipper"))
    }

    fn path() -> Option<PathBuf> {
        Self::dir().map(|d| d.join("settings.toml"))
    }

    /// Loads settings, falling back to defaults when the file is missing or
    /// invalid. The error, if any, is returned alongside for reporting.
    pub fn load() -> (Self, Option<anyhow::Error>) {
        let result = (|| -> Result<Option<Self>> {
            let Some(path) = Self::path() else {
                return Ok(None);
            };
            if !path.exists() {
                return Ok(None);
            }
            let text =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            let settings =
                toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
            Ok(Some(settings))
        })();

        let (mut settings, error) = match result {
            Ok(s) => (s.unwrap_or_default(), None),
            Err(e) => (Self::default(), Some(e)),
        };
        settings.sanitize();
        (settings, error)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path().context("no config directory")?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        fs::write(&path, toml::to_string_pretty(self)?)
            .with_context(|| format!("writing {}", path.display()))
    }
}

fn clamp_probability(p: f64) -> f64 {
    finite_or(p, 0.0).clamp(0.0, 1.0)
}

fn finite_or(v: f64, default: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_round_trip() {
        let s = Settings::default();
        let text = toml::to_string_pretty(&s).unwrap();
        assert_eq!(toml::from_str::<Settings>(&text).unwrap(), s);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let s: Settings = toml::from_str("action_mode = \"mouse\"").unwrap();
        assert_eq!(s.action_mode, ActionMode::Mouse);
        assert_eq!(s.skip_mode, SkipMode::Detect);
        assert_eq!(s.interval_min_ms, 120);
        assert_eq!(s.hotkey_exit, Some(VK_F10));
    }

    #[test]
    fn sanitize_fixes_inverted_ranges() {
        let mut s = Settings {
            interval_min_ms: 500,
            interval_max_ms: 100,
            break_min_secs: 10.0,
            break_max_secs: f64::NAN,
            slow_chance: 7.0,
            hotkey_exit: Some(keys::VK_F12),
            ..Settings::default()
        };
        s.sanitize();
        assert_eq!(s.interval_max_ms, 500);
        assert!(s.slow_interval_max_ms >= 500);
        assert_eq!(s.break_max_secs, 10.0);
        assert_eq!(s.slow_chance, 1.0);
        assert_eq!(s.hotkey_exit, None);
    }
}
