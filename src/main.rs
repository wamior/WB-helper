#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod config;
mod ocr;
mod printing;
mod capture;
mod updater;
mod security;
mod logger;


// We'll rename it to main.rs later or use it as main entry
// Actually eframe requires main.
fn main() -> eframe::Result<()> {
    security::anti_debug_check();
    updater::cleanup_old_executables();

    // Log functionality if needed, for now just print to stdout
    
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([600.0, 550.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ПВЗ Помощник",
        native_options,
        Box::new(|cc| Ok(Box::new(app::PvzApp::new(cc)))),
    )
}
