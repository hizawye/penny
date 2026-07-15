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
        toggle_id: MenuId,
        quit_id: MenuId,
    }

    impl Tray {
        /// Builds the tray icon, or `None` if the platform refuses it.
        pub fn new() -> Option<Self> {
            let toggle = MenuItem::new("Show / hide overlay", true, None);
            let quit = MenuItem::new("Quit Penny", true, None);

            let menu = Menu::new();
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
                toggle_id: toggle.id().clone(),
                quit_id: quit.id().clone(),
            })
        }

        /// Drains pending tray-menu clicks. Call once per frame.
        pub fn poll(&self) -> TrayAction {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
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

    /// A 32×32 filled circle in Penny's green (`#34c759`) used as the tray glyph.
    fn brand_icon() -> Icon {
        const N: i32 = 32;
        let mut rgba = vec![0u8; (N * N * 4) as usize];
        let center = (N as f32 - 1.0) / 2.0;
        let radius = center - 1.0;
        for y in 0..N {
            for x in 0..N {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                if dx * dx + dy * dy <= radius * radius {
                    let i = ((y * N + x) * 4) as usize;
                    rgba[i] = 0x34;
                    rgba[i + 1] = 0xc7;
                    rgba[i + 2] = 0x59;
                    rgba[i + 3] = 0xff;
                }
            }
        }
        Icon::from_rgba(rgba, N as u32, N as u32).expect("valid tray icon")
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
