//! Snapshot of the engine state shared with the UI.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use super::layout::Profile;
use crate::platform::{hotkeys::HotkeyAction, win32};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState {
    Paused,
    Running,
    Break { until: Instant },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    NotFound,
    Minimized,
    Found {
        width: i32,
        height: i32,
        focused: bool,
        profile: Profile,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub lines: u64,
    pub choices: u64,
    active: Duration,
    active_since: Option<Instant>,
}

impl Stats {
    pub fn active_time(&self, now: Instant) -> Duration {
        self.active + self.active_since.map_or(Duration::ZERO, |t| now - t)
    }

    fn set_active(&mut self, active: bool, now: Instant) {
        match (active, self.active_since) {
            (true, None) => self.active_since = Some(now),
            (false, Some(since)) => {
                self.active += now - since;
                self.active_since = None;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

/// Log messages are structured so that the UI can render them in its language.
#[derive(Clone, Debug, PartialEq)]
pub enum LogMsg {
    Started,
    Paused,
    GameFound { width: i32, height: i32 },
    GameLost,
    BreakStarted { secs: f64 },
    BreakEnded,
    HotkeyFailed { action: HotkeyAction, key: u16 },
    SettingsError(String),
    CaptureFailed(String),
    InputFailed(String),
}

impl LogMsg {
    pub fn level(&self) -> Level {
        match self {
            Self::HotkeyFailed { .. } | Self::SettingsError(_) | Self::CaptureFailed(_) => {
                Level::Warn
            }
            Self::InputFailed(_) => Level::Error,
            _ => Level::Info,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Local wall-clock time (h, m, s).
    pub time: (u16, u16, u16),
    pub msg: LogMsg,
    /// How many consecutive times the same message was logged.
    pub repeats: u32,
}

const LOG_CAPACITY: usize = 100;

#[derive(Debug)]
pub struct Status {
    pub run: RunState,
    pub game: GameStatus,
    pub stats: Stats,
    pub log: VecDeque<LogEntry>,
    /// Set when the exit hotkey is pressed; the UI closes the window.
    pub quit_requested: bool,
}

impl Status {
    pub fn log(&mut self, msg: LogMsg) {
        match msg.level() {
            Level::Info => log::info!("{msg:?}"),
            Level::Warn => log::warn!("{msg:?}"),
            Level::Error => log::error!("{msg:?}"),
        }
        let time = win32::local_time();
        if let Some(last) = self.log.back_mut() {
            if last.msg == msg {
                last.repeats += 1;
                last.time = time;
                return;
            }
        }
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(LogEntry {
            time,
            msg,
            repeats: 1,
        });
    }

    pub fn set_run(&mut self, run: RunState, now: Instant) {
        self.run = run;
        self.stats.set_active(run != RunState::Paused, now);
    }
}

#[derive(Clone)]
pub struct Shared {
    status: Arc<Mutex<Status>>,
    repaint: Arc<dyn Fn() + Send + Sync>,
}

impl Shared {
    pub fn new(repaint: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            status: Arc::new(Mutex::new(Status {
                run: RunState::Paused,
                game: GameStatus::NotFound,
                stats: Stats::default(),
                log: VecDeque::with_capacity(LOG_CAPACITY),
                quit_requested: false,
            })),
            repaint: Arc::new(repaint),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, Status> {
        // A panic while holding the lock cannot leave `Status` inconsistent
        // in a way that matters for display.
        self.status.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Mutates the status and asks the UI to redraw.
    pub fn update<T>(&self, f: impl FnOnce(&mut Status) -> T) -> T {
        let result = f(&mut self.lock());
        (self.repaint)();
        result
    }

    pub fn log(&self, msg: LogMsg) {
        self.update(|s| s.log(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_messages_are_collapsed() {
        let shared = Shared::new(|| {});
        shared.log(LogMsg::GameLost);
        shared.log(LogMsg::GameLost);
        shared.log(LogMsg::Started);
        let s = shared.lock();
        assert_eq!(s.log.len(), 2);
        assert_eq!(s.log[0].repeats, 2);
    }

    #[test]
    fn log_is_bounded() {
        let shared = Shared::new(|| {});
        for i in 0..(LOG_CAPACITY + 10) {
            shared.log(LogMsg::CaptureFailed(i.to_string()));
        }
        assert_eq!(shared.lock().log.len(), LOG_CAPACITY);
    }

    #[test]
    fn active_time_accumulates_only_while_running() {
        let t0 = Instant::now();
        let mut stats = Stats::default();
        stats.set_active(true, t0);
        stats.set_active(false, t0 + Duration::from_secs(5));
        stats.set_active(true, t0 + Duration::from_secs(100));
        assert_eq!(
            stats.active_time(t0 + Duration::from_secs(102)),
            Duration::from_secs(7)
        );
    }
}
