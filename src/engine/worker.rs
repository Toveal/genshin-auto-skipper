//! Background thread that watches the game and advances dialogues.

use std::{
    io,
    sync::mpsc::{self, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rand::rngs::ThreadRng;

use super::{
    detector::{self, DialogState},
    humanizer::Humanizer,
    layout::{Layout, Point},
    state::{GameStatus, LogMsg, RunState, Shared},
};
use crate::{
    game::{self, GameWindow},
    platform::win32,
    settings::{ActionMode, Settings, SkipMode},
};

#[derive(Debug)]
pub enum Command {
    Start,
    Pause,
    Apply(Settings),
    /// Stop the worker; `from_hotkey` also asks the UI to close.
    Quit {
        from_hotkey: bool,
    },
}

/// How often game state is refreshed while paused (for the UI).
const IDLE_POLL: Duration = Duration::from_secs(1);
/// Poll period while running but not in a dialogue (as in the reference).
const SCAN_POLL: Duration = Duration::from_millis(100);
/// Poll period while the game is running but not focused.
const UNFOCUSED_POLL: Duration = Duration::from_millis(250);

pub struct Worker {
    tx: Sender<Command>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn spawn(settings: Settings, shared: Shared) -> io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("worker".into())
            .spawn(move || {
                let mut engine = Engine::new(settings, shared);
                let mut timeout = Duration::ZERO;
                loop {
                    match rx.recv_timeout(timeout) {
                        Ok(cmd) => {
                            if !engine.handle(cmd) {
                                break;
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                    timeout = engine.tick(Instant::now());
                }
            })?;
        Ok(Self {
            tx,
            handle: Some(handle),
        })
    }

    pub fn sender(&self) -> Sender<Command> {
        self.tx.clone()
    }

    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.send(Command::Quit { from_hotkey: false });
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct Target {
    window: GameWindow,
    layout: Layout,
}

struct Engine {
    settings: Settings,
    shared: Shared,
    humanizer: Humanizer<ThreadRng>,
    running: bool,
    break_until: Option<Instant>,
    target: Option<Target>,
    next_action_at: Instant,
}

impl Engine {
    fn new(settings: Settings, shared: Shared) -> Self {
        let now = Instant::now();
        Self {
            settings,
            shared,
            humanizer: Humanizer::new(rand::rng(), now),
            running: false,
            break_until: None,
            target: None,
            next_action_at: now,
        }
    }

    /// Returns `false` when the worker must stop.
    fn handle(&mut self, cmd: Command) -> bool {
        let now = Instant::now();
        match cmd {
            Command::Start => self.set_running(true, now),
            Command::Pause => self.set_running(false, now),
            Command::Apply(settings) => self.settings = settings,
            Command::Quit { from_hotkey } => {
                if from_hotkey {
                    self.shared.update(|s| s.quit_requested = true);
                }
                return false;
            }
        }
        true
    }

    fn set_running(&mut self, running: bool, now: Instant) {
        if running == self.running {
            return;
        }
        self.running = running;
        self.break_until = None;
        self.next_action_at = now;
        self.humanizer.reset_break_timer(now);

        let (run, msg) = if running {
            (RunState::Running, LogMsg::Started)
        } else {
            (RunState::Paused, LogMsg::Paused)
        };
        self.shared.update(|s| {
            s.set_run(run, now);
            s.log(msg);
        });

        if self.settings.sound {
            // Rising tone for start, falling for pause.
            let (first, second) = if running { (660, 880) } else { (880, 520) };
            win32::beep(first, 60);
            win32::beep(second, 80);
        }
    }

    /// Does one step of work and returns how long to wait before the next.
    fn tick(&mut self, now: Instant) -> Duration {
        let game = self.refresh_game();

        if !self.running {
            return IDLE_POLL;
        }

        if let Some(until) = self.break_until {
            if now < until {
                return (until - now).min(IDLE_POLL);
            }
            self.break_until = None;
            self.next_action_at = now;
            self.shared.update(|s| {
                s.set_run(RunState::Running, now);
                s.log(LogMsg::BreakEnded);
            });
        }

        match game {
            GameStatus::Found { focused: true, .. } => {}
            GameStatus::Found { .. } => return UNFOCUSED_POLL,
            GameStatus::NotFound | GameStatus::Minimized => return IDLE_POLL / 2,
        }

        if now < self.next_action_at {
            return self.next_action_at - now;
        }

        let Some(target) = &self.target else {
            return IDLE_POLL / 2;
        };
        let state = match self.settings.skip_mode {
            // Every tick counts as a dialogue line: just press the key.
            SkipMode::Press => DialogState::Talking,
            SkipMode::Detect => match target.window.capture().and_then(|capture| {
                detector::detect(&capture, &target.layout, self.settings.color_tolerance)
            }) {
                Ok(state) => state,
                Err(e) => {
                    self.shared.log(LogMsg::CaptureFailed(e.to_string()));
                    return UNFOCUSED_POLL;
                }
            },
        };

        match state {
            DialogState::None | DialogState::Loading => SCAN_POLL,
            DialogState::Talking | DialogState::Choice => {
                if let Some(pause) = self.humanizer.maybe_break(&self.settings, now) {
                    let until = now + pause;
                    self.break_until = Some(until);
                    self.shared.update(|s| {
                        s.set_run(RunState::Break { until }, now);
                        s.log(LogMsg::BreakStarted {
                            secs: pause.as_secs_f64(),
                        });
                    });
                    return pause.min(IDLE_POLL);
                }

                let choice = state == DialogState::Choice;
                match self.act(choice) {
                    Ok(()) => self.shared.update(|s| {
                        if choice {
                            s.stats.choices += 1;
                        } else {
                            s.stats.lines += 1;
                        }
                    }),
                    Err(e) => self.shared.log(LogMsg::InputFailed(e.to_string())),
                }

                // Measured from the start of the press, so that the hold time
                // does not stretch the configured interval.
                self.next_action_at = now + self.humanizer.interval(&self.settings);
                self.next_action_at
                    .saturating_duration_since(Instant::now())
            }
        }
    }

    fn act(&mut self, choice: bool) -> io::Result<()> {
        let s = &self.settings;
        let hold = self.humanizer.hold(s);
        match (choice, s.action_mode) {
            (false, _) => game::tap_key(s.interact_key, hold),
            (true, ActionMode::Keyboard) => game::tap_key(s.confirm_key, hold),
            (true, ActionMode::Mouse) => {
                let Some(target) = &self.target else {
                    return Ok(());
                };
                let (min, max) = (target.layout.option_zone_min, target.layout.option_zone_max);
                let (x, y) = self.humanizer.point_in((min.x, min.y), (max.x, max.y));
                let (sx, sy) = target.window.client_to_screen(Point { x, y })?;
                game::click_at(sx, sy, hold)
            }
        }
    }

    /// Finds or re-validates the game window, recomputes the layout on
    /// resolution changes and publishes the result.
    fn refresh_game(&mut self) -> GameStatus {
        if self.target.as_ref().is_some_and(|t| !t.window.exists()) {
            self.target = None;
            self.shared.log(LogMsg::GameLost);
        }

        let window = match &self.target {
            Some(t) => t.window,
            None => match GameWindow::find() {
                Some(w) => w,
                None => return self.publish_game(GameStatus::NotFound),
            },
        };

        if window.is_minimized() {
            return self.publish_game(GameStatus::Minimized);
        }

        let (width, height) = match window.client_size() {
            Ok(size) if size.0 > 0 && size.1 > 0 => size,
            _ => return self.publish_game(GameStatus::Minimized),
        };

        let changed = self
            .target
            .as_ref()
            .is_none_or(|t| (t.layout.width, t.layout.height) != (width, height));
        if changed {
            self.target = Some(Target {
                window,
                layout: Layout::new(width, height),
            });
            self.shared.log(LogMsg::GameFound { width, height });
        }

        let profile = self.target.as_ref().map(|t| t.layout.profile);
        self.publish_game(GameStatus::Found {
            width,
            height,
            focused: window.is_foreground(),
            profile: profile.expect("target was just set"),
        })
    }

    fn publish_game(&self, game: GameStatus) -> GameStatus {
        if self.shared.lock().game != game {
            self.shared.update(|s| s.game = game);
        }
        game
    }
}
