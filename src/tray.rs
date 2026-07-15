//! Optional system-tray icon with a small menu to show/hide the overlay or
//! quit. Enabled on Windows and macOS; on Linux `tray-icon` needs GTK, so a
//! no-op stub is compiled instead and the menu simply never appears.

#[cfg(not(target_os = "linux"))]
pub use imp::Tray;

#[cfg(target_os = "linux")]
pub use stub::Tray;

/// What the user picked from the tray menu this frame. Identical on every
/// platform so the caller needs no `cfg`; the Linux stub only ever returns
/// `None`.
pub enum TrayAction {
    None,
    ToggleDraw,
    ToggleOverlay,
    Quit,
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::TrayAction;

    use tray_icon::{
        menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
        Icon, TrayIcon, TrayIconBuilder,
    };

    pub struct Tray {
        // Held for the process lifetime; dropping it removes the icon.
        _tray: TrayIcon,
        draw_id: MenuId,
        toggle_id: MenuId,
        quit_id: MenuId,
    }

    impl Tray {
        /// Builds the tray icon, or `None` if the platform refuses it.
        pub fn new() -> Option<Self> {
            // A hotkey-independent escape from click-through mode: the tray
            // menu keeps working even while the window passes the mouse
            // through, so the user is never stranded if Ctrl+Shift+D is
            // unavailable.
            let draw = MenuItem::new("Toggle draw / click-through", true, None);
            let toggle = MenuItem::new("Show / hide overlay", true, None);
            let quit = MenuItem::new("Quit Penny", true, None);

            let menu = Menu::new();
            menu.append(&draw).ok()?;
            menu.append(&toggle).ok()?;
            menu.append(&PredefinedMenuItem::separator()).ok()?;
            menu.append(&quit).ok()?;

            let tray = TrayIconBuilder::new()
                .with_tooltip("Penny — draw over any app")
                .with_icon(brand_icon())
                .with_menu(Box::new(menu))
                .build()
                .ok()?;

            Some(Self {
                _tray: tray,
                draw_id: draw.id().clone(),
                toggle_id: toggle.id().clone(),
                quit_id: quit.id().clone(),
            })
        }

        /// Drains pending tray-menu clicks. Call once per frame.
        pub fn poll(&self) -> TrayAction {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == self.draw_id {
                    return TrayAction::ToggleDraw;
                }
                if event.id == self.toggle_id {
                    return TrayAction::ToggleOverlay;
                }
                if event.id == self.quit_id {
                    return TrayAction::Quit;
                }
            }
            TrayAction::None
        }
    }

    /// The Penny brand logo, decoded from the embedded PNG and scaled to a
    /// 32×32 tray glyph.
    fn brand_icon() -> Icon {
        const N: u32 = 32;
        let img = image::load_from_memory(include_bytes!("../assets/penny.png"))
            .expect("decode brand icon")
            .resize_exact(N, N, image::imageops::FilterType::Lanczos3)
            .into_rgba8();
        Icon::from_rgba(img.into_raw(), N, N).expect("valid tray icon")
    }
}

#[cfg(target_os = "linux")]
mod stub {
    use super::TrayAction;

    pub struct Tray;

    impl Tray {
        pub fn new() -> Option<Self> {
            None
        }

        pub fn poll(&self) -> TrayAction {
            TrayAction::None
        }
    }
}
