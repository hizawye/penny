# Penny

Draw over any app while streaming — pen, highlighter, arrows, text, eraser, laser pointer. Hotkeys to toggle draw mode and click-through instantly. Works with OBS, Discord, Zoom, Teams. Cross-platform, built in Rust.

Penny is a transparent, always-on-top overlay for content creators: mark things, point stuff out, circle UI elements — over games, browsers, PowerPoint, IDEs, anything on screen.

## Features (v0.1)

**Drawing tools**
- ✏ Pen — freehand
- 🖊 Highlighter — thick, semi-transparent
- ╱ Line and ➤ Arrow
- ▭ Rectangle and ◯ Ellipse
- T Text — click to place, type, Enter to commit
- ⌫ Eraser — object mode (removes whole strokes)
- ● Laser pointer — glowing trail that fades, leaves no permanent mark

**Control**
- Click-through toggle: draw mode vs. passing clicks to the app underneath
- Global hotkeys (work even when another app has focus)
- Undo/redo, clear-all
- Auto-fade timer — marks disappear N seconds after you draw them
- Quick color palette on number keys 1–8
- Stroke width via scroll wheel, `[` / `]`, or the slider
- Shape snapping — hold **Shift** for perfect 45° lines, squares, and circles
- Export annotated screenshot to PNG (saved to your Pictures folder)
- Settings persist between sessions

## Hotkeys

| Global (system-wide) | |
|---|---|
| `Ctrl+Shift+D` | Toggle draw mode / click-through |
| `Ctrl+Shift+H` | Hide / show the overlay |
| `Ctrl+Shift+X` | Clear all annotations |

| In draw mode | |
|---|---|
| `P` `H` `N` `A` `R` `O` `T` `E` `L` | Pen, Highlighter, liNe, Arrow, Rect, ellipse (O), Text, Eraser, Laser |
| `1`–`8` | Quick colors (red, orange, yellow, green, cyan, blue, magenta, white) |
| `[` / `]` or scroll | Stroke width down / up |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / redo |
| `Delete` or `Backspace` | Clear all |
| `F` | Toggle auto-fade |
| `Ctrl+S` | Export annotated screenshot (PNG) |
| `Shift` + drag | Snap shapes (45° lines, squares, circles) |

## Build & run

```sh
cargo run --release
```

Requires Rust 1.80+. On Linux you'll want the usual GUI dev packages (X11/Wayland, `libxkbcommon`, OpenGL).

## Capture / streaming compatibility

Penny is a real top-most window, so anything that captures the desktop sees it:

- **OBS** — use *Display Capture* (or *Window Capture* of the target app plus Penny). *Game Capture* hooks the game process and won't see a separate overlay window.
- **Discord / Zoom / Teams screen share** — full-screen share captures the overlay; these use desktop-duplication APIs that include top-most windows.
- **Games** — windowed / borderless-windowed works. Exclusive-fullscreen games render below nothing, so switch the game to borderless. (True exclusive-fullscreen support would require renderer hooking — out of scope for v1.)
- **Wayland** — best effort: global hotkeys and always-on-top depend on the compositor; X11 is the reliable path on Linux.

## Roadmap

- Zoom-in overlay (magnify a region, ZoomIt-style)
- Spotlight / dim mode (darken everything except a focus area)
- Screen freeze (annotate a paused frame while the app keeps running)
- Countdown / timer overlay for streams
- Multi-monitor support (v1 covers the primary monitor)
- Pixel eraser mode
- Session presets (coding vs. gaming vs. teaching setups)
- Custom color presets + per-tool opacity

## License

MIT
