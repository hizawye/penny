//! System-wide hotkeys that work even when the overlay is click-through
//! or another app has focus.
//!
//! - Ctrl+Shift+D — toggle draw mode / click-through
//! - Ctrl+Shift+H — hide/show the overlay
//! - Ctrl+Shift+X — clear all annotations

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalAction {
    ToggleDraw,
    ToggleVisibility,
    ClearAll,
}

pub struct Hotkeys {
    /// Kept alive for the lifetime of the app; dropping it unregisters the hotkeys.
    _manager: Option<GlobalHotKeyManager>,
    toggle_draw: u32,
    toggle_visibility: u32,
    clear_all: u32,
}

impl Hotkeys {
    /// Registers global hotkeys. Failure is non-fatal (e.g. Wayland without
    /// a global-shortcuts portal): the overlay still works, just without
    /// system-wide shortcuts.
    pub fn register() -> Self {
        let mods = Modifiers::CONTROL | Modifiers::SHIFT;
        let toggle_draw = HotKey::new(Some(mods), Code::KeyD);
        let toggle_visibility = HotKey::new(Some(mods), Code::KeyH);
        let clear_all = HotKey::new(Some(mods), Code::KeyX);

        let ids = (toggle_draw.id(), toggle_visibility.id(), clear_all.id());

        let manager = GlobalHotKeyManager::new().ok().and_then(|m| {
            m.register_all(&[toggle_draw, toggle_visibility, clear_all])
                .ok()
                .map(|_| m)
        });
        if manager.is_none() {
            eprintln!(
                "penny: global hotkeys unavailable on this system; \
                 use the toolbar to toggle draw mode"
            );
        }

        Self {
            _manager: manager,
            toggle_draw: ids.0,
            toggle_visibility: ids.1,
            clear_all: ids.2,
        }
    }

    /// Drains pending hotkey presses.
    pub fn poll(&self) -> Vec<GlobalAction> {
        let mut actions = Vec::new();
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state != global_hotkey::HotKeyState::Pressed {
                continue;
            }
            if event.id == self.toggle_draw {
                actions.push(GlobalAction::ToggleDraw);
            } else if event.id == self.toggle_visibility {
                actions.push(GlobalAction::ToggleVisibility);
            } else if event.id == self.clear_all {
                actions.push(GlobalAction::ClearAll);
            }
        }
        actions
    }
}
