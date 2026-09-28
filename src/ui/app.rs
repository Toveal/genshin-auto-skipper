use std::time::{Duration, Instant};

use eframe::egui::{
    self, Color32, CornerRadius, DragValue, Margin, RichText, ScrollArea, Stroke, Vec2,
};

use super::i18n::{self, Texts};
use crate::{
    engine::{
        layout::Profile,
        state::{GameStatus, Level, LogEntry, LogMsg, RunState, Shared, Stats},
        worker::{Command, Worker},
    },
    platform::{
        hotkeys::{Bindings, HotkeyAction, HotkeyEvent, Hotkeys},
        keys, win32,
    },
    settings::{ActionMode, Language, Settings, SkipMode},
};

const GOLD: Color32 = Color32::from_rgb(236, 200, 120);
const GREEN: Color32 = Color32::from_rgb(98, 201, 128);
const AMBER: Color32 = Color32::from_rgb(232, 170, 72);
const BLUE: Color32 = Color32::from_rgb(112, 164, 240);
const RED: Color32 = Color32::from_rgb(232, 96, 96);
const CARD: Color32 = Color32::from_rgb(34, 38, 52);
const BG: Color32 = Color32::from_rgb(22, 24, 33);

/// Settings are written to disk once they stop changing for this long.
const SAVE_DELAY: Duration = Duration::from_millis(600);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyField {
    Interact,
    Confirm,
    Hotkey(HotkeyAction),
}

pub struct SkipperApp {
    settings: Settings,
    shared: Shared,
    // Dropped in declaration order: hotkeys first so that no command arrives
    // at a stopped worker.
    hotkeys: Hotkeys,
    worker: Worker,
    capturing: Option<KeyField>,
    /// The user tried to bind a key that cannot be a hotkey.
    reserved_key_tried: bool,
    elevated: bool,
    save_due: Option<Instant>,
}

impl SkipperApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings: Settings,
        settings_error: Option<anyhow::Error>,
    ) -> anyhow::Result<Self> {
        setup_style(&cc.egui_ctx);

        let ctx = cc.egui_ctx.clone();
        let shared = Shared::new(move || ctx.request_repaint());
        if let Some(e) = settings_error {
            shared.log(LogMsg::SettingsError(format!("{e:#}")));
        }

        let worker = Worker::spawn(settings.clone(), shared.clone())?;

        let tx = worker.sender();
        let hotkey_shared = shared.clone();
        let hotkeys = Hotkeys::spawn(bindings(&settings), move |event| match event {
            HotkeyEvent::Pressed(action) => {
                let _ = tx.send(match action {
                    HotkeyAction::Start => Command::Start,
                    HotkeyAction::Pause => Command::Pause,
                    HotkeyAction::Exit => Command::Quit { from_hotkey: true },
                });
            }
            HotkeyEvent::RegisterFailed(action, key) => {
                hotkey_shared.log(LogMsg::HotkeyFailed { action, key });
            }
        })?;

        Ok(Self {
            settings,
            shared,
            hotkeys,
            worker,
            capturing: None,
            reserved_key_tried: false,
            elevated: win32::is_elevated(),
            save_due: None,
        })
    }

    fn t(&self) -> &'static Texts {
        i18n::texts(self.settings.language)
    }

    fn settings_changed(&mut self, ctx: &egui::Context, old: &Settings) {
        self.settings.sanitize();
        self.worker.send(Command::Apply(self.settings.clone()));
        if self.capturing.is_none() && bindings(old) != bindings(&self.settings) {
            self.hotkeys.rebind(bindings(&self.settings));
        }
        if old.always_on_top != self.settings.always_on_top {
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if self.settings.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                },
            ));
        }
        self.save_due = Some(Instant::now() + SAVE_DELAY);
    }

    fn save_now(&mut self) {
        self.save_due = None;
        if let Err(e) = self.settings.save() {
            self.shared.log(LogMsg::SettingsError(format!("{e:#}")));
        }
    }

    fn start_capture(&mut self, field: KeyField) {
        self.capturing = Some(field);
        // Registered hotkeys swallow their keys, so release them while the
        // user picks a new one.
        self.hotkeys.rebind(Vec::new());
    }

    fn finish_capture(&mut self, vk: Option<u16>) {
        let Some(field) = self.capturing.take() else {
            return;
        };
        if let Some(vk) = vk {
            let s = &mut self.settings;
            match field {
                KeyField::Interact => s.interact_key = vk,
                KeyField::Confirm => s.confirm_key = vk,
                KeyField::Hotkey(action) => {
                    // A key can trigger only one action.
                    for slot in [&mut s.hotkey_start, &mut s.hotkey_pause, &mut s.hotkey_exit] {
                        if *slot == Some(vk) {
                            *slot = None;
                        }
                    }
                    *hotkey_slot(s, action) = Some(vk);
                }
            }
        }
        self.hotkeys.rebind(bindings(&self.settings));
    }

    fn handle_capture_input(&mut self, ctx: &egui::Context) {
        if self.capturing.is_none() {
            return;
        }
        let pressed = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key {
                    key, pressed: true, ..
                } => Some(*key),
                _ => None,
            })
        });
        match pressed {
            Some(egui::Key::Escape) => self.finish_capture(None),
            Some(key) => {
                let Some(vk) = keys::from_egui(key) else {
                    return;
                };
                let is_hotkey = matches!(self.capturing, Some(KeyField::Hotkey(_)));
                self.reserved_key_tried = is_hotkey && !keys::can_be_hotkey(vk);
                if !self.reserved_key_tried {
                    self.finish_capture(Some(vk));
                }
            }
            None => {}
        }
    }
}

impl eframe::App for SkipperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = Instant::now();
        // Snapshot before anything this frame can modify the settings.
        let old = self.settings.clone();

        self.handle_capture_input(&ctx);

        let (run, game, stats, log, quit) = {
            let s = self.shared.lock();
            (s.run, s.game, s.stats, s.log.clone(), s.quit_requested)
        };
        if quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::Frame::central_panel(ui.style())
            .fill(BG)
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    self.header(ui);
                    ui.add_space(8.0);
                    self.status_card(ui, run, game, now);
                    if !self.elevated {
                        ui.add_space(8.0);
                        banner(ui, RED, self.t().not_admin);
                    }
                    ui.add_space(8.0);
                    self.stats_row(ui, &stats, now);
                    ui.add_space(8.0);
                    self.help(ui);
                    self.settings_ui(ui);
                    self.log_ui(ui, &log);
                    ui.add_space(6.0);
                    self.footer(ui);
                });
            });

        if self.settings != old {
            self.settings_changed(&ctx, &old);
        }

        if let Some(due) = self.save_due {
            if now >= due {
                self.save_now();
            } else {
                ctx.request_repaint_after(due - now);
            }
        }
        if run != RunState::Paused {
            // Keep the timers ticking.
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.save_due.is_some() {
            self.save_now();
        }
    }
}

// ------------------------------------------------------------- sections ---

impl SkipperApp {
    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⏩").size(22.0).color(GOLD));
            ui.label(
                RichText::new("Genshin Auto Skipper")
                    .size(19.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let lang = &mut self.settings.language;
                ui.selectable_value(lang, Language::Ru, "RU");
                ui.selectable_value(lang, Language::En, "EN");
            });
        });
    }

    fn status_card(&mut self, ui: &mut egui::Ui, run: RunState, game: GameStatus, now: Instant) {
        let t = self.t();
        let (label, color) = match run {
            RunState::Paused => (t.status_paused.to_string(), AMBER),
            RunState::Running => (t.status_running.to_string(), GREEN),
            RunState::Break { until } => {
                let left = until.saturating_duration_since(now).as_secs_f32().ceil();
                (format!("{} · {left:.0} s", t.status_break), BLUE)
            }
        };

        card(ui, Some(color), |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 6.0, color);
                ui.label(RichText::new(label).size(22.0).strong().color(color));
            });

            let (game_text, game_color) = game_line(t, game);
            ui.label(RichText::new(game_text).color(game_color));
            if self.settings.skip_mode == SkipMode::Press {
                let s = &self.settings;
                let timing = if s.humanize {
                    format!("{}–{} {}", s.interval_min_ms, s.slow_interval_max_ms, t.ms)
                } else {
                    format!("{} {}", s.interval_min_ms, t.ms)
                };
                let text = format!(
                    "{}: {} · {timing}",
                    t.skip_mode_press,
                    keys::name(s.interact_key)
                );
                ui.label(RichText::new(text).color(BLUE).small());
            }
            ui.add_space(6.0);

            let running = run != RunState::Paused;
            let (text, key, fill) = if running {
                (
                    t.pause,
                    self.settings.hotkey_pause,
                    Color32::from_rgb(70, 64, 48),
                )
            } else {
                (
                    t.start,
                    self.settings.hotkey_start,
                    Color32::from_rgb(48, 84, 62),
                )
            };
            let text = match key {
                Some(vk) => format!("{text}  ({})", keys::name(vk)),
                None => text.to_string(),
            };
            let button = egui::Button::new(RichText::new(text).size(16.0).strong())
                .fill(fill)
                .min_size(Vec2::new(ui.available_width(), 38.0));
            if ui.add(button).clicked() {
                self.worker.send(if running {
                    Command::Pause
                } else {
                    Command::Start
                });
            }
        });
    }

    fn stats_row(&self, ui: &mut egui::Ui, stats: &Stats, now: Instant) {
        let t = self.t();
        let secs = stats.active_time(now).as_secs();
        let time = format!("{}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60);
        let items = [
            (
                stats.lines.to_string(),
                if self.settings.skip_mode == SkipMode::Press {
                    t.stat_presses
                } else {
                    t.stat_lines
                },
            ),
            (stats.choices.to_string(), t.stat_choices),
            (time, t.stat_time),
        ];
        ui.columns(3, |cols| {
            for (col, (value, caption)) in cols.iter_mut().zip(items) {
                card(col, None, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(value)
                                .size(18.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.label(RichText::new(caption).small().weak());
                    });
                });
            }
        });
    }

    fn help(&self, ui: &mut egui::Ui) {
        let t = self.t();
        egui::CollapsingHeader::new(RichText::new(t.help).strong())
            .id_salt("help")
            .default_open(true)
            .show(ui, |ui| {
                for (i, step) in t.help_steps.iter().enumerate() {
                    ui.horizontal_top(|ui| {
                        ui.label(RichText::new(format!("{}.", i + 1)).color(GOLD));
                        ui.add(egui::Label::new(*step).wrap());
                    });
                }
            });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        egui::CollapsingHeader::new(RichText::new(t.settings).strong())
            .id_salt("settings")
            .show(ui, |ui| {
                section(ui, t.group_actions);
                egui::Grid::new("actions")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(t.skip_mode);
                        ui.vertical(|ui| {
                            let mode = &mut self.settings.skip_mode;
                            ui.radio_value(mode, SkipMode::Detect, t.skip_mode_detect)
                                .on_hover_text(t.skip_mode_detect_hint);
                            ui.radio_value(mode, SkipMode::Press, t.skip_mode_press)
                                .on_hover_text(t.skip_mode_press_hint);
                        });
                        ui.end_row();

                        ui.label(t.interact_key);
                        self.key_button(ui, KeyField::Interact);
                        ui.end_row();

                        if self.settings.skip_mode == SkipMode::Press {
                            return;
                        }

                        ui.label(t.mode);
                        ui.vertical(|ui| {
                            let mode = &mut self.settings.action_mode;
                            ui.radio_value(mode, ActionMode::Keyboard, t.mode_keyboard);
                            ui.radio_value(mode, ActionMode::Mouse, t.mode_mouse);
                        });
                        ui.end_row();

                        if self.settings.action_mode == ActionMode::Keyboard {
                            ui.label(t.confirm_key);
                            self.key_button(ui, KeyField::Confirm);
                            ui.end_row();
                        }
                    });

                section(ui, t.group_hotkeys);
                egui::Grid::new("hotkeys")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        for (action, label) in [
                            (HotkeyAction::Start, t.hotkey_start),
                            (HotkeyAction::Pause, t.hotkey_pause),
                            (HotkeyAction::Exit, t.hotkey_exit),
                        ] {
                            ui.label(label);
                            ui.horizontal(|ui| {
                                self.key_button(ui, KeyField::Hotkey(action));
                                let slot = hotkey_slot(&mut self.settings, action);
                                if slot.is_some()
                                    && ui.small_button("✖").on_hover_text(t.clear).clicked()
                                {
                                    *slot = None;
                                }
                            });
                            ui.end_row();
                        }
                    });
                if self.reserved_key_tried {
                    ui.label(RichText::new(t.hotkey_reserved).color(AMBER).small());
                }
                if has_conflict(&self.settings) {
                    ui.label(RichText::new(t.hotkey_conflict).color(AMBER).small());
                }

                section(ui, t.group_timing);
                let s = &mut self.settings;
                ui.checkbox(&mut s.humanize, t.humanize)
                    .on_hover_text(t.humanize_hint);
                ui.add_enabled_ui(s.humanize, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(t.interval);
                        ui.add(DragValue::new(&mut s.interval_min_ms).range(30..=5000));
                        ui.label("–");
                        ui.add(DragValue::new(&mut s.interval_max_ms).range(30..=5000));
                    });
                    ui.checkbox(&mut s.breaks, t.breaks)
                        .on_hover_text(t.breaks_hint);
                    ui.add_enabled_ui(s.breaks, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t.break_length);
                            ui.add(
                                DragValue::new(&mut s.break_min_secs)
                                    .range(0.5..=600.0)
                                    .speed(0.1)
                                    .max_decimals(1),
                            );
                            ui.label("–");
                            ui.add(
                                DragValue::new(&mut s.break_max_secs)
                                    .range(0.5..=600.0)
                                    .speed(0.1)
                                    .max_decimals(1),
                            );
                        });
                    });
                });

                section(ui, t.group_other);
                ui.horizontal(|ui| {
                    ui.label(t.tolerance);
                    ui.add(egui::Slider::new(&mut s.color_tolerance, 0..=32))
                        .on_hover_text(t.tolerance_hint);
                });
                ui.checkbox(&mut s.sound, t.sound);
                ui.checkbox(&mut s.always_on_top, t.always_on_top);

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button(t.reset).clicked() {
                        let language = self.settings.language;
                        self.settings = Settings {
                            language,
                            ..Settings::default()
                        };
                    }
                    if ui.button(t.open_folder).clicked() {
                        open_settings_folder();
                    }
                });
            });
    }

    fn key_button(&mut self, ui: &mut egui::Ui, field: KeyField) {
        let t = self.t();
        let current = match field {
            KeyField::Interact => Some(self.settings.interact_key),
            KeyField::Confirm => Some(self.settings.confirm_key),
            KeyField::Hotkey(action) => *hotkey_slot(&mut self.settings, action),
        };
        let active = self.capturing == Some(field);
        let text = if active {
            RichText::new(t.press_key).color(GOLD)
        } else {
            RichText::new(current.map_or_else(|| t.unbound.to_string(), keys::name)).strong()
        };
        let button = egui::Button::new(text)
            .min_size(Vec2::new(64.0, 0.0))
            .selected(active);
        if ui.add(button).clicked() {
            if active {
                self.finish_capture(None);
            } else {
                if self.capturing.is_some() {
                    self.finish_capture(None);
                }
                self.start_capture(field);
            }
        }
    }

    fn log_ui(&self, ui: &mut egui::Ui, log: &std::collections::VecDeque<LogEntry>) {
        let t = self.t();
        egui::CollapsingHeader::new(RichText::new(t.log).strong())
            .id_salt("log")
            .show(ui, |ui| {
                if log.is_empty() {
                    ui.label(RichText::new(t.log_empty).weak());
                    return;
                }
                ScrollArea::vertical()
                    .max_height(170.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in log {
                            let color = match entry.msg.level() {
                                Level::Info => ui.visuals().text_color(),
                                Level::Warn => AMBER,
                                Level::Error => RED,
                            };
                            let (h, m, s) = entry.time;
                            let mut text = i18n::log_message(self.settings.language, &entry.msg);
                            if entry.repeats > 1 {
                                text.push_str(&format!("  ×{}", entry.repeats));
                            }
                            ui.horizontal_top(|ui| {
                                ui.label(
                                    RichText::new(format!("{h:02}:{m:02}:{s:02}"))
                                        .monospace()
                                        .weak(),
                                );
                                ui.add(egui::Label::new(RichText::new(text).color(color)).wrap());
                            });
                        }
                    });
            });
    }

    fn footer(&self, ui: &mut egui::Ui) {
        let t = self.t();
        let s = &self.settings;
        let parts: Vec<String> = [
            (s.hotkey_start, t.hotkey_start),
            (s.hotkey_pause, t.hotkey_pause),
            (s.hotkey_exit, t.hotkey_exit),
        ]
        .into_iter()
        .filter_map(|(key, label)| key.map(|k| format!("{} — {}", keys::name(k), label)))
        .collect();
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(parts.join("   ·   ")).small().weak());
            ui.label(
                RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .small()
                    .weak(),
            );
        });
    }
}

// -------------------------------------------------------------- helpers ---

fn game_line(t: &Texts, game: GameStatus) -> (String, Color32) {
    match game {
        GameStatus::NotFound => (t.game_not_found.into(), RED),
        GameStatus::Minimized => (t.game_minimized.into(), AMBER),
        GameStatus::Found {
            width,
            height,
            focused,
            profile,
        } => {
            let mut text = format!(
                "Genshin Impact · {width}×{height} · {}",
                if focused {
                    t.game_focused
                } else {
                    t.game_unfocused
                }
            );
            match profile {
                Profile::Standard => {}
                Profile::Wide => text.push_str(&format!(" · {}", t.profile_wide)),
                Profile::Measured => text.push_str(&format!(" · {}", t.profile_measured)),
            }
            (text, if focused { GREEN } else { AMBER })
        }
    }
}

fn hotkey_slot(s: &mut Settings, action: HotkeyAction) -> &mut Option<u16> {
    match action {
        HotkeyAction::Start => &mut s.hotkey_start,
        HotkeyAction::Pause => &mut s.hotkey_pause,
        HotkeyAction::Exit => &mut s.hotkey_exit,
    }
}

fn game_keys(s: &Settings) -> Vec<u16> {
    let mut keys = vec![s.interact_key];
    if s.skip_mode == SkipMode::Detect && s.action_mode == ActionMode::Keyboard {
        keys.push(s.confirm_key);
    }
    keys
}

fn has_conflict(s: &Settings) -> bool {
    let game = game_keys(s);
    [s.hotkey_start, s.hotkey_pause, s.hotkey_exit]
        .into_iter()
        .flatten()
        .any(|k| game.contains(&k))
}

/// Hotkeys to register. Keys the game needs are skipped: a registered hotkey
/// would swallow them.
fn bindings(s: &Settings) -> Bindings {
    let game = game_keys(s);
    [
        (HotkeyAction::Start, s.hotkey_start),
        (HotkeyAction::Pause, s.hotkey_pause),
        (HotkeyAction::Exit, s.hotkey_exit),
    ]
    .into_iter()
    .map(|(action, key)| (action, key.filter(|k| !game.contains(k))))
    .collect()
}

fn open_settings_folder() {
    if let Some(dir) = Settings::dir() {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::process::Command::new("explorer").arg(dir).spawn();
    }
}

fn card<R>(ui: &mut egui::Ui, accent: Option<Color32>, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let stroke = accent.map_or(Stroke::NONE, |c| Stroke::new(1.0, c.gamma_multiply(0.5)));
    egui::Frame::new()
        .fill(CARD)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

fn banner(ui: &mut egui::Ui, color: Color32, text: &str) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.15))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(egui::Label::new(RichText::new(text).color(color)).wrap());
        });
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(title).color(GOLD).strong());
    ui.separator();
}

fn setup_style(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(10.0, 4.0);
        style.visuals.selection.bg_fill = GOLD.gamma_multiply(0.45);
        style.visuals.hyperlink_color = GOLD;
        for (text_style, size) in [
            (egui::TextStyle::Body, 14.0),
            (egui::TextStyle::Button, 14.0),
            (egui::TextStyle::Small, 11.5),
            (egui::TextStyle::Monospace, 12.5),
        ] {
            if let Some(font) = style.text_styles.get_mut(&text_style) {
                font.size = size;
            }
        }
    });
}
