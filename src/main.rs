//! Penny — draw over any app while streaming.
//!
//! A transparent, always-on-top annotation overlay: pen, highlighter,
//! lines/arrows, shapes, text, eraser, and a laser pointer, with global
//! hotkeys and a click-through mode so the app underneath stays usable.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hotkeys;
mod model;
mod toolbar;
mod tray;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Penny")
            .with_app_id("penny-overlay")
            .with_transparent(true)
            .with_decorations(false)
            .with_resizable(false)
            .with_window_level(egui::WindowLevel::AlwaysOnTop)
            .with_position(egui::pos2(0.0, 0.0))
            // Resized to the full monitor once it is known (first frame).
            .with_inner_size(egui::vec2(1280.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Penny",
        options,
        Box::new(|cc| Ok(Box::new(app::PennyApp::new(cc)))),
    )
}
