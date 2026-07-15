//! Core data types: tools, marks, palette, and persisted settings.

use egui::{Color32, Pos2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    Pen,
    Highlighter,
    Line,
    Arrow,
    Rect,
    Ellipse,
    Text,
    Eraser,
    Laser,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Tool::Pen => "Pen",
            Tool::Highlighter => "Highlighter",
            Tool::Line => "Line",
            Tool::Arrow => "Arrow",
            Tool::Rect => "Rectangle",
            Tool::Ellipse => "Ellipse",
            Tool::Text => "Text",
            Tool::Eraser => "Eraser",
            Tool::Laser => "Laser",
        }
    }

    /// Phosphor icon glyph for this tool (see `egui_phosphor::regular`).
    pub fn icon(self) -> &'static str {
        use egui_phosphor::regular as ph;
        match self {
            Tool::Pen => ph::PENCIL_SIMPLE,
            Tool::Highlighter => ph::HIGHLIGHTER,
            Tool::Line => ph::LINE_SEGMENT,
            Tool::Arrow => ph::ARROW_UP_RIGHT,
            Tool::Rect => ph::RECTANGLE,
            Tool::Ellipse => ph::CIRCLE,
            Tool::Text => ph::TEXT_T,
            Tool::Eraser => ph::ERASER,
            Tool::Laser => ph::DOT,
        }
    }

    pub fn hotkey(self) -> &'static str {
        match self {
            Tool::Pen => "P",
            Tool::Highlighter => "H",
            Tool::Line => "N",
            Tool::Arrow => "A",
            Tool::Rect => "R",
            Tool::Ellipse => "O",
            Tool::Text => "T",
            Tool::Eraser => "E",
            Tool::Laser => "L",
        }
    }

    pub const ALL: [Tool; 9] = [
        Tool::Pen,
        Tool::Highlighter,
        Tool::Line,
        Tool::Arrow,
        Tool::Rect,
        Tool::Ellipse,
        Tool::Text,
        Tool::Eraser,
        Tool::Laser,
    ];
}

/// A committed annotation on the canvas.
#[derive(Debug, Clone)]
pub enum MarkKind {
    /// Freehand polyline (pen or highlighter — translucency is baked into the color).
    Path { points: Vec<Pos2> },
    Line { from: Pos2, to: Pos2 },
    Arrow { from: Pos2, to: Pos2 },
    Rect { from: Pos2, to: Pos2 },
    Ellipse { from: Pos2, to: Pos2 },
    Text { pos: Pos2, text: String },
}

#[derive(Debug, Clone)]
pub struct Mark {
    pub kind: MarkKind,
    pub color: Color32,
    pub width: f32,
    /// Time (egui clock, seconds) the mark was committed — used by the auto-fade timer.
    pub created: f64,
}

impl Mark {
    /// Rough hit test used by the object eraser.
    pub fn hit(&self, p: Pos2, radius: f32) -> bool {
        let r = radius + self.width * 0.5;
        match &self.kind {
            MarkKind::Path { points } => segments_hit(points, p, r),
            MarkKind::Line { from, to } | MarkKind::Arrow { from, to } => {
                dist_to_segment(p, *from, *to) <= r
            }
            MarkKind::Rect { from, to } => {
                let rect = egui::Rect::from_two_pos(*from, *to);
                let corners = [
                    rect.left_top(),
                    rect.right_top(),
                    rect.right_bottom(),
                    rect.left_bottom(),
                    rect.left_top(),
                ];
                segments_hit(&corners, p, r)
            }
            MarkKind::Ellipse { from, to } => {
                let rect = egui::Rect::from_two_pos(*from, *to);
                let c = rect.center();
                let (a, b) = (rect.width() * 0.5, rect.height() * 0.5);
                if a < 1.0 || b < 1.0 {
                    return rect.expand(r).contains(p);
                }
                // Distance from the ellipse outline, approximated on the normalized radius.
                let nx = (p.x - c.x) / a;
                let ny = (p.y - c.y) / b;
                let d = (nx * nx + ny * ny).sqrt();
                (d - 1.0).abs() * a.min(b) <= r
            }
            MarkKind::Text { pos, text } => {
                let approx = egui::Rect::from_min_size(
                    *pos,
                    egui::vec2(text.chars().count() as f32 * self.width * 0.6, self.width * 1.4),
                );
                approx.expand(r).contains(p)
            }
        }
    }
}

fn segments_hit(points: &[Pos2], p: Pos2, r: f32) -> bool {
    if points.len() == 1 {
        return points[0].distance(p) <= r;
    }
    points.windows(2).any(|w| dist_to_segment(p, w[0], w[1]) <= r)
}

fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    if len2 <= f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Quick-swap palette bound to number keys 1–8.
pub const PALETTE: [Color32; 8] = [
    Color32::from_rgb(0xff, 0x3b, 0x30), // 1 red
    Color32::from_rgb(0xff, 0x95, 0x00), // 2 orange
    Color32::from_rgb(0xff, 0xd6, 0x0a), // 3 yellow
    Color32::from_rgb(0x34, 0xc7, 0x59), // 4 green
    Color32::from_rgb(0x32, 0xad, 0xe6), // 5 cyan
    Color32::from_rgb(0x0a, 0x84, 0xff), // 6 blue
    Color32::from_rgb(0xbf, 0x5a, 0xf2), // 7 magenta
    Color32::WHITE,                      // 8 white
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub color_idx: usize,
    pub stroke_width: f32,
    pub highlighter_opacity: u8,
    pub fade_enabled: bool,
    pub fade_secs: f32,
    pub text_size: f32,
    pub tool: Tool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color_idx: 0,
            stroke_width: 4.0,
            highlighter_opacity: 90,
            fade_enabled: false,
            fade_secs: 5.0,
            text_size: 28.0,
            tool: Tool::Pen,
        }
    }
}

impl Settings {
    pub fn color(&self) -> Color32 {
        PALETTE[self.color_idx.min(PALETTE.len() - 1)]
    }

    fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("penny").join("settings.json"))
    }

    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(p) = Self::path() {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(s) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, s);
            }
        }
    }
}
