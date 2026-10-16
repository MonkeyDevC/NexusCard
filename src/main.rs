#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod afc;
mod airlift;
mod airtraffic;
mod app;
mod apple;
mod card_info;
mod device;
mod flasher;
mod image_skin;
mod i18n;
mod i18n_es;
mod license;
mod paths;
mod scanner;
mod ui_kit;
mod wallet_backup;

fn main() -> eframe::Result<()> {
    paths::migrate_legacy_data();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(window_icon())
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([940.0, 640.0])
            .with_title(concat!("NexusCard v", env!("CARGO_PKG_VERSION"))),
        ..Default::default()
    };

    eframe::run_native(
        concat!("NexusCard v", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|cc| Ok(Box::new(app::NexusCardApp::new(cc)))),
    )
}

/// Window / taskbar icon, decoded from the embedded NexusCard logo.
fn window_icon() -> egui::IconData {
    match image::load_from_memory(include_bytes!("../assets/icon-256.png")) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            egui::IconData { width: rgba.width(), height: rgba.height(), rgba: rgba.into_raw() }
        }
        Err(_) => egui::IconData::default(),
    }
}
