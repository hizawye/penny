//! Floating toolbar: tool buttons, palette, stroke width, fade timer,
//! click-through toggle, undo/redo/clear/export.

use egui::{Color32, RichText};
use egui_phosphor::regular as ph;

use crate::app::PennyApp;
use crate::model::{Tool, PALETTE};

pub fn show(app: &mut PennyApp, ctx: &egui::Context) {
    let frame = egui::Frame::window(&ctx.style())
        .fill(Color32::from_rgba_unmultiplied(20, 20, 24, 235))
        .corner_radius(10.0);

    egui::Window::new("penny_toolbar")
        .title_bar(false)
        .resizable(false)
        .frame(frame)
        .default_pos(egui::pos2(24.0, 24.0))
        .show(ctx, |ui| {
            if app.click_through {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{} click-through", ph::HAND_POINTING))
                            .color(Color32::from_gray(180))
                            .small(),
                    );
                    // Point at whichever escape actually works on this machine.
                    let hint = if app.draw_hotkey_available() {
                        "Ctrl+Shift+D to draw"
                    } else if app.has_tray() {
                        "tray menu → Toggle draw"
                    } else {
                        "restart to draw"
                    };
                    ui.label(RichText::new(hint).small().weak());
                });
                return;
            }

            // The draw toggle is the only hotkey that escapes click-through.
            // If it failed to register, warn before the user gets stranded.
            if !app.draw_hotkey_available() {
                let msg = if app.has_tray() {
                    "⚠ Ctrl+Shift+D unavailable — use the tray menu to leave click-through"
                } else {
                    "⚠ Ctrl+Shift+D unavailable — no way back from click-through"
                };
                ui.label(
                    RichText::new(msg)
                        .small()
                        .color(Color32::from_rgb(0xff, 0xb3, 0x4d)),
                );
                ui.add_space(4.0);
            }

            // Tools
            ui.horizontal(|ui| {
                for tool in Tool::ALL {
                    let selected = app.settings.tool == tool;
                    let btn = egui::Button::new(RichText::new(tool.icon()).size(16.0))
                        .min_size(egui::vec2(30.0, 30.0))
                        .fill(if selected {
                            Color32::from_rgb(0x0a, 0x84, 0xff)
                        } else {
                            Color32::from_rgba_unmultiplied(255, 255, 255, 12)
                        });
                    let r = ui
                        .add(btn)
                        .on_hover_text(format!("{} ({})", tool.label(), tool.hotkey()));
                    if r.clicked() {
                        app.set_tool(tool);
                    }
                }
            });

            ui.add_space(4.0);

            // Palette
            ui.horizontal(|ui| {
                for (i, color) in PALETTE.iter().enumerate() {
                    let selected = app.settings.color_idx == i;
                    let size = egui::vec2(22.0, 22.0);
                    let (rect, r) = ui.allocate_exact_size(size, egui::Sense::click());
                    let p = ui.painter();
                    p.circle_filled(rect.center(), 9.0, *color);
                    if selected {
                        p.circle_stroke(
                            rect.center(),
                            11.0,
                            egui::Stroke::new(2.0, Color32::WHITE),
                        );
                    }
                    if r.on_hover_text(format!("color {}", i + 1)).clicked() {
                        app.settings.color_idx = i;
                        app.mark_settings_dirty();
                    }
                }
            });

            ui.add_space(4.0);

            // Stroke width + fade
            ui.horizontal(|ui| {
                ui.label(RichText::new("width").small().weak());
                if ui
                    .add(egui::Slider::new(&mut app.settings.stroke_width, 1.0..=40.0))
                    .changed()
                {
                    app.mark_settings_dirty();
                }
            });
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut app.settings.fade_enabled, RichText::new("auto-fade").small())
                    .changed()
                {
                    app.mark_settings_dirty();
                }
                if app.settings.fade_enabled
                    && ui
                        .add(
                            egui::Slider::new(&mut app.settings.fade_secs, 1.0..=30.0)
                                .suffix("s"),
                        )
                        .changed()
                {
                    app.mark_settings_dirty();
                }
            });

            ui.add_space(4.0);

            // Actions
            ui.horizontal(|ui| {
                if ui.button(ph::ARROW_ARC_LEFT).on_hover_text("Undo (Ctrl+Z)").clicked() {
                    app.undo();
                }
                if ui.button(ph::ARROW_ARC_RIGHT).on_hover_text("Redo (Ctrl+Shift+Z)").clicked() {
                    app.redo();
                }
                if ui.button(ph::TRASH).on_hover_text("Clear all (Del / Ctrl+Shift+X)").clicked() {
                    app.clear_all();
                }
                if ui
                    .button(ph::CAMERA)
                    .on_hover_text("Export annotated screenshot (Ctrl+S)")
                    .clicked()
                {
                    app.request_export(ctx);
                }
                // Only offer click-through when there's a way back out of it,
                // so the toolbar button can never strand the user.
                let escapable = app.draw_hotkey_available() || app.has_tray();
                let hint = if escapable {
                    "Click-through: pass clicks to the app underneath (Ctrl+Shift+D)"
                } else {
                    "Click-through unavailable: no hotkey or tray to return from it"
                };
                if ui
                    .add_enabled(escapable, egui::Button::new(ph::HAND_POINTING))
                    .on_hover_text(hint)
                    .clicked()
                {
                    app.set_click_through(ctx, true);
                }
            });

            if let Some(path) = &app.last_export {
                ui.label(RichText::new(format!("saved {path}")).small().weak());
            }
        });
}
