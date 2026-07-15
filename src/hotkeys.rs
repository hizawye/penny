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
    /// `None` if this shortcut could not be claimed (e.g. another app owns it).
    toggle_draw: Option<u32>,
    toggle_visibility: Option<u32>,
    clear_all: Option<u32>,
}

impl Hotkeys {
    /// Registers global hotkeys. Failure is non-fatal (e.g. Wayland without
    /// a global-shortcuts portal): the overlay still works, just without
    /// system-wide shortcuts.
    ///
    /// Each shortcut is registered independently so that one conflict — say
    /// another running app already owning Ctrl+Shift+D — cannot knock out the
    /// others. This matters most for `toggle_draw`, the only hotkey that can
    /// escape click-through mode.
    pub fn register() -> Self {
        let mods = Modifiers::CONTROL | Modifiers::SHIFT;
        let manager = GlobalHotKeyManager::new().ok();

        let register_one = |code: Code| -> Option<u32> {
            let manager = manager.as_ref()?;
            let hotkey = HotKey::new(Some(mods), code);
            manager.register(hotkey).ok().map(|_| hotkey.id())
        };

        let toggle_draw = register_one(Code::KeyD);
        let toggle_visibility = register_one(Code::KeyH);
        let clear_all = register_one(Code::KeyX);

        if toggle_draw.is_none() {
            eprintln!(
                "penny: could not register Ctrl+Shift+D (draw toggle); \
                 use the tray menu to leave click-through mode"
            );
        }

        Self {
            _manager: manager,
            toggle_draw,
            toggle_visibility,
            clear_all,
        }
    }

    /// Whether the draw-mode toggle (Ctrl+Shift+D) is armed. When false, the
    /// hotkey cannot rescue the user from click-through mode, so the UI must
    /// offer another way out.
    pub fn toggle_draw_available(&self) -> bool {
        self.toggle_draw.is_some()
    }

    /// Drains pending hotkey presses.
    pub fn poll(&self) -> Vec<GlobalAction> {
        let mut actions = Vec::new();
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state != global_hotkey::HotKeyState::Pressed {
                continue;
            }
            if Some(event.id) == self.toggle_draw {
                actions.push(GlobalAction::ToggleDraw);
            } else if Some(event.id) == self.toggle_visibility {
                actions.push(GlobalAction::ToggleVisibility);
            } else if Some(event.id) == self.clear_all {
                actions.push(GlobalAction::ClearAll);
            }
        }
        actions
    }
}
