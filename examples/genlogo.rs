//! Logo generator for Penny — the single source of truth for the app icon.
//!
//! Renders a rounded "app tile" with a bold marker-stroke "P" (the pen) in
//! Penny's brand green, plus a glowing laser-dot pen tip. Everything is drawn
//! procedurally with 4× supersampling so it stays crisp from 16 px to 1024 px.
//!
//! Run with:  cargo run --release --example genlogo
//! Writes PNGs and a multi-size `penny.ico` into `assets/`.

use std::fs;
use std::io::Write;
use std::path::Path;

use image::{ImageEncoder, RgbaImage};

// ---- palette ---------------------------------------------------------------

// Dark tile, matching the toolbar's `rgb(20,20,24)` chrome.
const BG_TOP: [f32; 3] = [30.0, 30.0, 38.0];
const BG_BOT: [f32; 3] = [14.0, 14.0, 18.0];

// Brand green (`#34c759`) as a subtle top→bottom gradient along the stroke.
const INK_TOP: [f32; 3] = [0x53 as f32, 0xe6 as f32, 0x88 as f32];
const INK_BOT: [f32; 3] = [0x22 as f32, 0xa8 as f32, 0x48 as f32];
const GLOW: [f32; 3] = [0x34 as f32, 0xc7 as f32, 0x59 as f32];

// ---- geometry (all in 0..1 tile space) -------------------------------------

const CORNER: f32 = 0.225; // rounded-rect radius
const STROKE_W: f32 = 0.115; // marker thickness

// "P": a vertical stem plus a right-side bowl on the top half.
const STEM_X: f32 = 0.345;
const STEM_TOP: f32 = 0.215;
const STEM_BOT: f32 = 0.805;
const BOWL_R: f32 = 0.150;
// Bowl centre so its left edge meets the stem; spans the top half.
const BOWL_CX: f32 = STEM_X + BOWL_R;
const BOWL_CY: f32 = STEM_TOP + BOWL_R;

// Glowing laser-dot pen tip at the bottom of the stem.
const TIP_X: f32 = STEM_X;
const TIP_Y: f32 = STEM_BOT;
const TIP_R: f32 = 0.058;

// ---- tiny sdf/vec helpers --------------------------------------------------

fn dist_seg(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (abx, aby) = (bx - ax, by - ay);
    let (apx, apy) = (px - ax, py - ay);
    let t = ((apx * abx + apy * aby) / (abx * abx + aby * aby)).clamp(0.0, 1.0);
    let (cx, cy) = (ax + abx * t, ay + aby * t);
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

/// Distance to the right-half arc (a "C" opening left) of the bowl circle.
fn dist_bowl(px: f32, py: f32) -> f32 {
    let (dx, dy) = (px - BOWL_CX, py - BOWL_CY);
    if dx >= 0.0 {
        ((dx * dx + dy * dy).sqrt() - BOWL_R).abs()
    } else {
        // Off the arc — measure to the nearer endpoint (top or bottom).
        let ey = if dy < 0.0 { -BOWL_R } else { BOWL_R };
        (dx * dx + (dy - ey).powi(2)).sqrt()
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Straight-alpha source-over of `src` (rgb + a) onto `dst`.
fn over(dst: &mut [f32; 4], rgb: [f32; 3], a: f32) {
    let out_a = a + dst[3] * (1.0 - a);
    if out_a <= 1e-6 {
        return;
    }
    for i in 0..3 {
        dst[i] = (rgb[i] * a + dst[i] * dst[3] * (1.0 - a)) / out_a;
    }
    dst[3] = out_a;
}

// ---- rendering -------------------------------------------------------------

/// Renders the icon at `size` px using `SS`× supersampling.
fn render(size: u32) -> RgbaImage {
    const SS: u32 = 4;
    let hi = size * SS;
    let n = hi as f32;
    // Anti-alias width in tile units: ~1 hi-res texel.
    let aa = 1.2 / n;

    let mut acc = vec![[0.0f32; 4]; (size * size) as usize];

    for hy in 0..hi {
        for hx in 0..hi {
            // Tile-space coords, pixel centre.
            let u = (hx as f32 + 0.5) / n;
            let v = (hy as f32 + 0.5) / n;

            let mut px = [0.0f32, 0.0, 0.0, 0.0];

            // 1. Rounded tile.
            let (qx, qy) = ((u - 0.5).abs() - (0.5 - CORNER), (v - 0.5).abs() - (0.5 - CORNER));
            let tile_sd = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - CORNER;
            let tile_cov = smoothstep(aa, -aa, tile_sd);
            if tile_cov > 0.0 {
                let bg = lerp3(BG_TOP, BG_BOT, v);
                over(&mut px, mul(bg, 1.0 / 255.0), tile_cov);
                // Faint top rim highlight for definition.
                let rim = smoothstep(aa, -aa, (tile_sd + 0.012).abs() - 0.006);
                let rim_light = (1.0 - v) * 0.30;
                over(&mut px, [1.0, 1.0, 1.0], rim * rim_light * tile_cov);
            }

            // 2. Soft green glow beneath the stroke (streaming/laser vibe).
            let stroke_sd = dist_seg(u, v, STEM_X, STEM_TOP, STEM_X, STEM_BOT)
                .min(dist_bowl(u, v))
                - STROKE_W * 0.5;
            let glow = (1.0 - smoothstep(0.0, 0.11, stroke_sd.max(0.0))) * 0.45 * tile_cov;
            if glow > 0.0 {
                over(&mut px, mul(GLOW, 1.0 / 255.0), glow);
            }

            // 3. The marker "P".
            let ink_cov = smoothstep(aa, -aa, stroke_sd) * tile_cov;
            if ink_cov > 0.0 {
                let mut ink = lerp3(INK_TOP, INK_BOT, smoothstep(STEM_TOP, STEM_BOT, v));
                // Inner highlight along the upper-left of the stroke.
                let hl = (1.0 - smoothstep(-STROKE_W * 0.5, 0.0, stroke_sd)) * 0.0;
                ink = lerp3(ink, [255.0, 255.0, 255.0], hl);
                over(&mut px, mul(ink, 1.0 / 255.0), ink_cov);
            }

            // 4. Glowing laser-dot pen tip: white core → green halo.
            let td = ((u - TIP_X).powi(2) + (v - TIP_Y).powi(2)).sqrt();
            let halo = (1.0 - smoothstep(0.0, TIP_R * 2.2, td)) * 0.55 * tile_cov;
            if halo > 0.0 {
                over(&mut px, mul(GLOW, 1.0 / 255.0), halo);
            }
            let core = smoothstep(TIP_R, TIP_R - aa * 2.0, td) * tile_cov;
            if core > 0.0 {
                over(&mut px, [0.94, 1.0, 0.96], core);
            }

            // Accumulate into the low-res box.
            let (lx, ly) = (hx / SS, hy / SS);
            let a = &mut acc[(ly * size + lx) as usize];
            for i in 0..4 {
                a[i] += px[i];
            }
        }
    }

    let mut img = RgbaImage::new(size, size);
    let inv = 1.0 / (SS * SS) as f32;
    for (i, p) in acc.iter().enumerate() {
        let a = p[3] * inv;
        // De-average colour (colour was stored straight, weighted equally).
        let r = (p[0] * inv).clamp(0.0, 1.0);
        let g = (p[1] * inv).clamp(0.0, 1.0);
        let b = (p[2] * inv).clamp(0.0, 1.0);
        let px = image::Rgba([
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
            (a.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]);
        img.put_pixel((i as u32) % size, (i as u32) / size, px);
    }
    img
}

fn mul(c: [f32; 3], k: f32) -> [f32; 3] {
    [c[0] * k, c[1] * k, c[2] * k]
}

fn png_bytes(img: &RgbaImage) -> Vec<u8> {
    let mut buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buf)
        .write_image(img.as_raw(), img.width(), img.height(), image::ExtendedColorType::Rgba8)
        .expect("encode png");
    buf
}

/// Assembles a Windows .ico whose entries are PNG-compressed (Vista+).
fn write_ico(path: &Path, imgs: &[(u32, Vec<u8>)]) {
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]); // reserved, type=icon
    out.extend_from_slice(&(imgs.len() as u16).to_le_bytes());
    let mut offset = 6 + imgs.len() * 16;
    for (size, png) in imgs {
        let dim = if *size >= 256 { 0u8 } else { *size as u8 };
        out.push(dim); // width
        out.push(dim); // height
        out.push(0); // palette
        out.push(0); // reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // colour planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bpp
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in imgs {
        out.extend_from_slice(png);
    }
    fs::File::create(path).unwrap().write_all(&out).unwrap();
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    fs::create_dir_all(&dir).unwrap();

    // Full-size marketing/source renders.
    for size in [1024u32, 512, 256] {
        let img = render(size);
        img.save(dir.join(format!("penny-{size}.png"))).unwrap();
    }
    // Canonical name used by the app for its window/tray icon.
    render(256).save(dir.join("penny.png")).unwrap();

    // Multi-size .ico for Windows (exe + installer + shortcuts).
    let ico_sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let entries: Vec<(u32, Vec<u8>)> =
        ico_sizes.iter().map(|&s| (s, png_bytes(&render(s)))).collect();
    write_ico(&dir.join("penny.ico"), &entries);

    println!("wrote assets/penny.png, penny-{{256,512,1024}}.png, penny.ico");
}
