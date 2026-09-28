// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod game;
mod platform;
mod settings;
mod ui;

use std::fs::{self, File};

use platform::win32::{self, SingleInstance};
use settings::{Language, Settings};

fn main() {
    win32::enable_dpi_awareness();
    init_logging();
    install_panic_hook();

    if let Err(e) = run() {
        log::error!("{e:#}");
        let t = ui::i18n::texts(system_language());
        win32::message_box(t.fatal, &format!("{e:#}"), true);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let (settings, settings_error) = Settings::load();

    let Some(_instance) = SingleInstance::acquire("Local\\genshin-auto-skipper")? else {
        let t = ui::i18n::texts(settings.language);
        win32::message_box("Genshin Auto Skipper", t.already_running, false);
        return Ok(());
    };

    ui::run(settings, settings_error)
}

fn system_language() -> Language {
    if win32::ui_language_is_russian() {
        Language::Ru
    } else {
        Language::En
    }
}

/// Logs to `app.log` next to the settings; the file is recreated every run.
fn init_logging() {
    let Some(dir) = Settings::dir() else { return };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(file) = File::create(dir.join("app.log")) {
        let _ = simplelog::WriteLogger::init(
            log::LevelFilter::Info,
            simplelog::Config::default(),
            file,
        );
    }
}

fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{info}");
        let t = ui::i18n::texts(system_language());
        win32::message_box(t.fatal, &info.to_string(), true);
        default(info);
    }));
}
