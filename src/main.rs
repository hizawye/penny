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

/// Decodes the embedded brand PNG into an eframe window icon. Falls back to no
/// icon if decoding ever fails (it shouldn't — the asset ships in the binary).
fn window_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../assets/penny.png"))
        .ok()?
        .into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Penny")
        .with_app_id("penny-overlay")
        .with_transparent(true);
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport
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
