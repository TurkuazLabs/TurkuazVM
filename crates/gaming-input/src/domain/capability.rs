// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/domain/capability.rs
// # 📌 Amac: Gaming Input runtime capability raporunu tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Editor, raw input, ADB fallback, Guest Agent multi-touch ve gamepad destek seviyelerini typed olarak tasir
// # Bagimli Oldugu Katman: Service | View

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamingInputCapabilities {
    pub profile_editor: bool,
    pub raw_keyboard: bool,
    pub raw_mouse_motion: bool,
    pub adb_single_touch_fallback: bool,
    pub persistent_multi_touch: bool,
    pub virtual_joystick: bool,
    pub relative_mouse_look: bool,
    pub host_gamepad_observation: bool,
}

impl GamingInputCapabilities {
    pub const fn foundation() -> Self {
        Self {
            profile_editor: true,
            raw_keyboard: true,
            raw_mouse_motion: true,
            adb_single_touch_fallback: true,
            persistent_multi_touch: false,
            virtual_joystick: true,
            relative_mouse_look: true,
            host_gamepad_observation: true,
        }
    }

    pub const fn with_persistent_multi_touch(mut self, available: bool) -> Self {
        self.persistent_multi_touch = available;
        self
    }
}
