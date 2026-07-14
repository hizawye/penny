//! The overlay application: a transparent, always-on-top, full-screen window
//! that renders annotations above everything else.

use std::time::Duration;

use egui::{Color32, Pos2, Stroke, ViewportCommand};

use crate::hotkeys::{GlobalAction, Hotkeys};
use crate::model::{Mark, MarkKind, Settings, Tool, PALETTE};
use crate::toolbar;

/// Laser trail points live this long (seconds).
const LASER_TRAIL_SECS: f64 = 0.8;
/// Seconds a faded mark takes to ramp from opaque to gone once `fade_secs` elapses.
const FADE_RAMP_SECS: f64 = 1.0;
const MAX_UNDO: usize = 100;

pub struct PennyApp {
    pub settings: Settings,
    marks: Vec<Mark>,
    undo_stack: Vec<Vec<Mark>>,
    redo_stack: Vec<Vec<Mark>>,
    /// In-progress freehand points while the pointer is down.
    active_path: Vec<Pos2>,
    /// Drag anchor for line/arrow/rect/ellipse.
    drag_from: Option<Pos2>,
    /// Live pointer position of an unfinished shape drag.
    drag_to: Option<Pos2>,
    /// Pending text entry: position + buffer being typed.
    pending_text: Option<(Pos2, String)>,
    laser_trail: Vec<(Pos2, f64)>,
    /// When true the window ignores the mouse and clicks reach the app underneath.
    pub click_through: bool,
    pub overlay_hidden: bool,
    hotkeys: Hotkeys,
    sized_to_monitor: bool,
    /// Path of the last exported screenshot, shown briefly in the toolbar.
    pub last_export: Option<String>,
    settings_dirty: bool,
    /// Shift state captured each frame, used by live snap previews.
    shift_down: bool,
}

impl PennyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            settings: Settings::load(),
            marks: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_path: Vec::new(),
            drag_from: None,
            drag_to: None,
            pending_text: None,
            laser_trail: Vec::new(),
            click_through: false,
            overlay_hidden: false,
            hotkeys: Hotkeys::register(),
            sized_to_monitor: false,
            last_export: None,
            settings_dirty: false,
            shift_down: false,
        }
    }

    fn snapshot(&mut self) {
        self.undo_stack.push(self.marks.clone());
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(std::mem::replace(&mut self.marks, prev));
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(std::mem::replace(&mut self.marks, next));
        }
    }

    pub fn clear_all(&mut self) {
        if !self.marks.is_empty() {
            self.snapshot();
            self.marks.clear();
        }
        self.active_path.clear();
        self.drag_from = None;
        self.drag_to = None;
        self.pending_text = None;
    }

    pub fn set_click_through(&mut self, ctx: &egui::Context, on: bool) {
        self.click_through = on;
        ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(on));
        if !on {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }

    pub fn set_tool(&mut self, tool: Tool) {
        if self.settings.tool != tool {
            self.settings.tool = tool;
            self.settings_dirty = true;
        }
        // Abort any half-finished shape when switching tools.
        self.active_path.clear();
        self.drag_from = None;
        self.drag_to = None;
    }

    pub fn mark_settings_dirty(&mut self) {
        self.settings_dirty = true;
    }

    /// Effective stroke color for the current tool (highlighter is translucent).
    fn stroke_color(&self) -> Color32 {
        let c = self.settings.color();
        if self.settings.tool == Tool::Highlighter {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), self.settings.highlighter_opacity)
        } else {
            c
        }
    }

    fn stroke_width(&self) -> f32 {
        if self.settings.tool == Tool::Highlighter {
            self.settings.stroke_width * 3.5
        } else {
            self.settings.stroke_width
        }
    }

    fn commit(&mut self, kind: MarkKind, now: f64) {
        self.snapshot();
        self.marks.push(Mark {
            kind,
            color: self.stroke_color(),
            width: match self.settings.tool {
                Tool::Text => self.settings.text_size,
                _ => self.stroke_width(),
            },
            created: now,
        });
    }

    // ------------------------------------------------------------------
    // Input
    // ------------------------------------------------------------------

    fn handle_global_hotkeys(&mut self, ctx: &egui::Context) {
        for action in self.hotkeys.poll() {
            match action {
                GlobalAction::ToggleDraw => {
                    let on = !self.click_through;
                    self.set_click_through(ctx, on);
                }
                GlobalAction::ToggleVisibility => {
                    self.overlay_hidden = !self.overlay_hidden;
                    // Keep receiving events; just stop painting and pass clicks through.
                    if self.overlay_hidden {
                        self.set_click_through(ctx, true);
                    }
                }
                GlobalAction::ClearAll => self.clear_all(),
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        // While typing a text annotation, keys belong to the text box.
        if self.pending_text.is_some() {
            return;
        }
        let (keys, mods, scroll) = ctx.input(|i| {
            let pressed: Vec<egui::Key> = [
                egui::Key::P,
                egui::Key::H,
                egui::Key::N,
                egui::Key::A,
                egui::Key::R,
                egui::Key::O,
                egui::Key::T,
                egui::Key::E,
                egui::Key::L,
                egui::Key::Z,
                egui::Key::Y,
                egui::Key::S,
                egui::Key::F,
                egui::Key::Delete,
                egui::Key::Backspace,
                egui::Key::OpenBracket,
                egui::Key::CloseBracket,
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
                egui::Key::Num7,
                egui::Key::Num8,
            ]
            .into_iter()
            .filter(|k| i.key_pressed(*k))
            .collect();
            (pressed, i.modifiers, i.raw_scroll_delta.y)
        });

        for key in keys {
            match key {
                egui::Key::P if mods.is_none() => self.set_tool(Tool::Pen),
                egui::Key::H if mods.is_none() => self.set_tool(Tool::Highlighter),
                egui::Key::N if mods.is_none() => self.set_tool(Tool::Line),
                egui::Key::A if mods.is_none() => self.set_tool(Tool::Arrow),
                egui::Key::R if mods.is_none() => self.set_tool(Tool::Rect),
                egui::Key::O if mods.is_none() => self.set_tool(Tool::Ellipse),
                egui::Key::T if mods.is_none() => self.set_tool(Tool::Text),
                egui::Key::E if mods.is_none() => self.set_tool(Tool::Eraser),
                egui::Key::L if mods.is_none() => self.set_tool(Tool::Laser),
                egui::Key::F if mods.is_none() => {
                    self.settings.fade_enabled = !self.settings.fade_enabled;
                    self.settings_dirty = true;
                }
                egui::Key::Z if mods.command && mods.shift => self.redo(),
                egui::Key::Z if mods.command => self.undo(),
                egui::Key::Y if mods.command => self.redo(),
                egui::Key::S if mods.command => self.request_export(ctx),
                egui::Key::Delete | egui::Key::Backspace if mods.is_none() => self.clear_all(),
                egui::Key::OpenBracket => {
                    self.settings.stroke_width = (self.settings.stroke_width - 1.0).max(1.0);
                    self.settings_dirty = true;
                }
                egui::Key::CloseBracket => {
                    self.settings.stroke_width = (self.settings.stroke_width + 1.0).min(40.0);
                    self.settings_dirty = true;
                }
                egui::Key::Num1
                | egui::Key::Num2
                | egui::Key::Num3
                | egui::Key::Num4
                | egui::Key::Num5
                | egui::Key::Num6
                | egui::Key::Num7
                | egui::Key::Num8
                    if mods.is_none() =>
                {
                    let idx = key as usize - egui::Key::Num1 as usize;
                    if idx < PALETTE.len() {
                        self.settings.color_idx = idx;
                        self.settings_dirty = true;
                    }
                }
                _ => {}
            }
        }

        // Scroll wheel adjusts stroke width (unless the pointer is over the toolbar).
        if scroll.abs() > 0.0 && !ctx.is_pointer_over_area() {
            let delta = if scroll > 0.0 { 1.0 } else { -1.0 };
            self.settings.stroke_width = (self.settings.stroke_width + delta).clamp(1.0, 40.0);
            self.settings_dirty = true;
        }
    }

    fn handle_drawing(&mut self, ctx: &egui::Context, response: &egui::Response, now: f64) {
        let pos = response.interact_pointer_pos();
        let hover = ctx.input(|i| i.pointer.hover_pos());

        // Laser leaves a fading trail while the pointer moves — no permanent mark.
        if self.settings.tool == Tool::Laser {
            if let Some(p) = hover {
                if self
                    .laser_trail
                    .last()
                    .is_none_or(|(last, _)| last.distance(p) > 1.0)
                {
                    self.laser_trail.push((p, now));
                }
            }
            return;
        }

        match self.settings.tool {
            Tool::Pen | Tool::Highlighter => {
                if response.drag_started() {
                    self.active_path.clear();
                }
                if response.dragged() {
                    if let Some(p) = pos {
                        if self
                            .active_path
                            .last()
                            .is_none_or(|last| last.distance(p) > 0.5)
                        {
                            self.active_path.push(p);
                        }
                    }
                }
                if response.drag_stopped() && !self.active_path.is_empty() {
                    let points = std::mem::take(&mut self.active_path);
                    self.commit(MarkKind::Path { points }, now);
                }
            }
            Tool::Line | Tool::Arrow | Tool::Rect | Tool::Ellipse => {
                if response.drag_started() {
                    self.drag_from = pos;
                }
                if response.dragged() {
                    self.drag_to = pos;
                }
                if response.drag_stopped() {
                    if let (Some(from), Some(to)) = (self.drag_from.take(), self.drag_to.take()) {
                        if from.distance(to) > 2.0 {
                            let to = self.maybe_snap(ctx, from, to);
                            let kind = match self.settings.tool {
                                Tool::Line => MarkKind::Line { from, to },
                                Tool::Arrow => MarkKind::Arrow { from, to },
                                Tool::Rect => MarkKind::Rect { from, to },
                                Tool::Ellipse => MarkKind::Ellipse { from, to },
                                _ => unreachable!(),
                            };
                            self.commit(kind, now);
                        }
                    }
                }
            }
            Tool::Text => {
                if response.clicked() {
                    if let Some(p) = pos {
                        self.pending_text = Some((p, String::new()));
                    }
                }
            }
            Tool::Eraser => {
                if response.dragged() || response.clicked() {
                    if let Some(p) = pos {
                        let radius = self.settings.stroke_width.max(8.0);
                        if self.marks.iter().any(|m| m.hit(p, radius)) {
                            self.snapshot();
                            self.marks.retain(|m| !m.hit(p, radius));
                        }
                    }
                }
            }
            Tool::Laser => unreachable!(),
        }
    }

    /// Shape snapping: hold Shift for perfectly horizontal/vertical/45° lines,
    /// squares, and circles.
    fn maybe_snap(&self, ctx: &egui::Context, from: Pos2, to: Pos2) -> Pos2 {
        if !ctx.input(|i| i.modifiers.shift) {
            return to;
        }
        snap_endpoint(self.settings.tool, from, to)
    }

    fn commit_pending_text(&mut self, now: f64) {
        if let Some((pos, text)) = self.pending_text.take() {
            if !text.trim().is_empty() {
                let tool = self.settings.tool;
                self.settings.tool = Tool::Text;
                self.commit(MarkKind::Text { pos, text }, now);
                self.settings.tool = tool;
            }
        }
    }

    // ------------------------------------------------------------------
    // Painting
    // ------------------------------------------------------------------

    fn fade_alpha(&self, created: f64, now: f64) -> f32 {
        if !self.settings.fade_enabled {
            return 1.0;
        }
        let age = now - created;
        let ramp_start = self.settings.fade_secs as f64;
        if age <= ramp_start {
            1.0
        } else {
            (1.0 - (age - ramp_start) / FADE_RAMP_SECS).max(0.0) as f32
        }
    }

    fn paint_mark(painter: &egui::Painter, mark: &Mark, alpha: f32) {
        let color = mark.color.gamma_multiply(alpha);
        let stroke = Stroke::new(mark.width, color);
        match &mark.kind {
            MarkKind::Path { points } => {
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(points.clone(), stroke));
                } else if let Some(p) = points.first() {
                    painter.circle_filled(*p, mark.width * 0.5, color);
                }
            }
            MarkKind::Line { from, to } => {
                painter.line_segment([*from, *to], stroke);
            }
            MarkKind::Arrow { from, to } => {
                Self::paint_arrow(painter, *from, *to, stroke);
            }
            MarkKind::Rect { from, to } => {
                let rect = egui::Rect::from_two_pos(*from, *to);
                painter.rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Middle);
            }
            MarkKind::Ellipse { from, to } => {
                let rect = egui::Rect::from_two_pos(*from, *to);
                painter.add(egui::Shape::Ellipse(egui::epaint::EllipseShape {
                    center: rect.center(),
                    radius: rect.size() * 0.5,
                    fill: Color32::TRANSPARENT,
                    stroke,
                }));
            }
            MarkKind::Text { pos, text } => {
                // Soft shadow so text stays readable over any background.
                painter.text(
                    *pos + egui::vec2(1.5, 1.5),
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(mark.width),
                    Color32::from_black_alpha((160.0 * alpha) as u8),
                );
                painter.text(
                    *pos,
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(mark.width),
                    color,
                );
            }
        }
    }

    fn paint_arrow(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
        painter.line_segment([from, to], stroke);
        let dir = (to - from).normalized();
        let head_len = (stroke.width * 4.0).max(12.0);
        let left = egui::vec2(
            dir.x * (-0.866) - dir.y * (-0.5),
            dir.x * (-0.5) + dir.y * (-0.866),
        );
        let right = egui::vec2(
            dir.x * (-0.866) - dir.y * 0.5,
            dir.x * 0.5 + dir.y * (-0.866),
        );
        painter.line_segment([to, to + left * head_len], stroke);
        painter.line_segment([to, to + right * head_len], stroke);
    }

    fn paint_live_preview(&self, painter: &egui::Painter, now: f64) {
        let stroke = Stroke::new(self.stroke_width(), self.stroke_color());

        if self.active_path.len() >= 2 {
            painter.add(egui::Shape::line(self.active_path.clone(), stroke));
        }
        if let (Some(from), Some(to)) = (self.drag_from, self.drag_to) {
            let to = match self.settings.tool {
                Tool::Line | Tool::Arrow | Tool::Rect | Tool::Ellipse => {
                    self.maybe_snap_preview(from, to)
                }
                _ => to,
            };
            match self.settings.tool {
                Tool::Line => {
                    painter.line_segment([from, to], stroke);
                }
                Tool::Arrow => Self::paint_arrow(painter, from, to, stroke),
                Tool::Rect => {
                    let rect = egui::Rect::from_two_pos(from, to);
                    painter.rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Middle);
                }
                Tool::Ellipse => {
                    let rect = egui::Rect::from_two_pos(from, to);
                    painter.add(egui::Shape::Ellipse(egui::epaint::EllipseShape {
                        center: rect.center(),
                        radius: rect.size() * 0.5,
                        fill: Color32::TRANSPARENT,
                        stroke,
                    }));
                }
                _ => {}
            }
        }

        // Laser trail: glowing dots that fade out.
        for (p, t) in &self.laser_trail {
            let age = now - t;
            let alpha = (1.0 - age / LASER_TRAIL_SECS).clamp(0.0, 1.0) as f32;
            if alpha > 0.0 {
                let c = self.settings.color();
                painter.circle_filled(
                    *p,
                    6.0 * alpha + 2.0,
                    c.gamma_multiply(alpha * 0.9),
                );
                painter.circle_filled(
                    *p,
                    12.0 * alpha + 4.0,
                    c.gamma_multiply(alpha * 0.25),
                );
            }
        }
    }

    fn maybe_snap_preview(&self, from: Pos2, to: Pos2) -> Pos2 {
        if self.shift_down {
            snap_endpoint(self.settings.tool, from, to)
        } else {
            to
        }
    }

    // ------------------------------------------------------------------
    // Export
    // ------------------------------------------------------------------

    pub fn request_export(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
    }

    fn handle_screenshots(&mut self, ctx: &egui::Context) {
        let images: Vec<std::sync::Arc<egui::ColorImage>> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        for img in images {
            let dir = dirs::picture_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let name = format!("penny-{}.png", chrono::Local::now().format("%Y%m%d-%H%M%S"));
            let path = dir.join(name);
            let [w, h] = img.size;
            let mut rgba = Vec::with_capacity(w * h * 4);
            for px in &img.pixels {
                rgba.extend_from_slice(&px.to_array());
            }
            if let Some(buf) = image::RgbaImage::from_raw(w as u32, h as u32, rgba) {
                if buf.save(&path).is_ok() {
                    self.last_export = Some(path.display().to_string());
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Housekeeping
    // ------------------------------------------------------------------

    fn ensure_fullscreen_size(&mut self, ctx: &egui::Context) {
        if self.sized_to_monitor {
            return;
        }
        if let Some(size) = ctx.input(|i| i.viewport().monitor_size) {
            if size.x > 1.0 && size.y > 1.0 {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::ZERO));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
                self.sized_to_monitor = true;
            }
        }
    }

    fn prune(&mut self, now: f64) {
        self.laser_trail.retain(|(_, t)| now - t < LASER_TRAIL_SECS);
        if self.settings.fade_enabled {
            let deadline = self.settings.fade_secs as f64 + FADE_RAMP_SECS;
            self.marks.retain(|m| now - m.created < deadline);
        }
    }
}

/// Snap a drag endpoint: lines/arrows to 45° increments, rects/ellipses to
/// squares/circles.
fn snap_endpoint(tool: Tool, from: Pos2, to: Pos2) -> Pos2 {
    let d = to - from;
    match tool {
        Tool::Line | Tool::Arrow => {
            let angle = d.y.atan2(d.x);
            let step = std::f32::consts::FRAC_PI_4;
            let snapped = (angle / step).round() * step;
            from + egui::vec2(snapped.cos(), snapped.sin()) * d.length()
        }
        Tool::Rect | Tool::Ellipse => {
            let side = d.x.abs().max(d.y.abs());
            from + egui::vec2(side * d.x.signum(), side * d.y.signum())
        }
        _ => to,
    }
}

impl eframe::App for PennyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // fully transparent window
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        self.shift_down = ctx.input(|i| i.modifiers.shift);

        self.ensure_fullscreen_size(ctx);
        self.handle_global_hotkeys(ctx);
        self.handle_screenshots(ctx);
        self.prune(now);

        if !self.click_through && !self.overlay_hidden {
            self.handle_keys(ctx);
        }

        let panel = egui::CentralPanel::default().frame(egui::Frame::NONE);
        panel.show(ctx, |ui| {
            if self.overlay_hidden {
                return;
            }

            let rect = ui.max_rect();
            let response = ui.interact(rect, ui.id().with("canvas"), egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);

            if !self.click_through {
                self.handle_drawing(ctx, &response, now);
            }

            // Committed marks.
            for mark in &self.marks {
                let alpha = self.fade_alpha(mark.created, now);
                if alpha > 0.0 {
                    Self::paint_mark(&painter, mark, alpha);
                }
            }

            self.paint_live_preview(&painter, now);

            // Pending text entry box.
            if let Some((pos, _)) = self.pending_text {
                let mut commit = false;
                let mut cancel = false;
                egui::Area::new(egui::Id::new("penny_text_entry"))
                    .fixed_pos(pos)
                    .show(ctx, |ui| {
                        if let Some((_, buf)) = &mut self.pending_text {
                            let edit = egui::TextEdit::singleline(buf)
                                .font(egui::FontId::proportional(self.settings.text_size))
                                .text_color(self.settings.color())
                                .hint_text("type, Enter to place")
                                .desired_width(320.0);
                            let r = ui.add(edit);
                            r.request_focus();
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                commit = true;
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                cancel = true;
                            }
                        }
                    });
                if commit {
                    self.commit_pending_text(now);
                } else if cancel {
                    self.pending_text = None;
                }
            }

            // Border hint: green when drawing, subtle when click-through.
            let border = if self.click_through {
                Stroke::new(1.0, Color32::from_white_alpha(20))
            } else {
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(0x34, 0xc7, 0x59, 180))
            };
            painter.rect_stroke(rect.shrink(1.0), 0.0, border, egui::StrokeKind::Inside);
        });

        if !self.overlay_hidden {
            toolbar::show(self, ctx);
        }

        if self.settings_dirty {
            self.settings.save();
            self.settings_dirty = false;
        }

        // Keep animating fades / laser trails, and keep polling global hotkeys
        // even when idle (they arrive on a channel, not as window events).
        let animating = !self.laser_trail.is_empty()
            || (self.settings.fade_enabled && !self.marks.is_empty())
            || self.drag_from.is_some()
            || !self.active_path.is_empty();
        ctx.request_repaint_after(if animating {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(100)
        });
    }
}
