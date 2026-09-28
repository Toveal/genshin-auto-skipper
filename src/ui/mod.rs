mod app;
pub mod i18n;

use std::sync::Arc;

use eframe::egui;

use crate::settings::Settings;

const ICON_RGBA: &[u8] = include_bytes!("../../assets/icon_64.rgba");

pub fn run(settings: Settings, settings_error: Option<anyhow::Error>) -> anyhow::Result<()> {
    let icon = egui::IconData {
        rgba: ICON_RGBA.to_vec(),
        width: 64,
        height: 64,
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Genshin Auto Skipper")
        .with_inner_size([380.0, 640.0])
        .with_min_inner_size([340.0, 360.0])
        .with_icon(Arc::new(icon))
        .with_window_level(if settings.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        });

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Genshin Auto Skipper",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::SkipperApp::new(
                cc,
                settings,
                settings_error,
            )?))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
