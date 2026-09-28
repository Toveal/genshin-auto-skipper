//! Keys that can be bound in the settings, as Win32 virtual-key codes.

use eframe::egui::Key;

/// (virtual-key code, display name, egui key)
const KEYS: &[(u16, &str, Key)] = &[
    (0x08, "Backspace", Key::Backspace),
    (0x09, "Tab", Key::Tab),
    (0x0D, "Enter", Key::Enter),
    (0x1B, "Esc", Key::Escape),
    (0x20, "Space", Key::Space),
    (0x21, "PgUp", Key::PageUp),
    (0x22, "PgDn", Key::PageDown),
    (0x23, "End", Key::End),
    (0x24, "Home", Key::Home),
    (0x25, "Left", Key::ArrowLeft),
    (0x26, "Up", Key::ArrowUp),
    (0x27, "Right", Key::ArrowRight),
    (0x28, "Down", Key::ArrowDown),
    (0x2D, "Insert", Key::Insert),
    (0x2E, "Delete", Key::Delete),
    (0x30, "0", Key::Num0),
    (0x31, "1", Key::Num1),
    (0x32, "2", Key::Num2),
    (0x33, "3", Key::Num3),
    (0x34, "4", Key::Num4),
    (0x35, "5", Key::Num5),
    (0x36, "6", Key::Num6),
    (0x37, "7", Key::Num7),
    (0x38, "8", Key::Num8),
    (0x39, "9", Key::Num9),
    (0x41, "A", Key::A),
    (0x42, "B", Key::B),
    (0x43, "C", Key::C),
    (0x44, "D", Key::D),
    (0x45, "E", Key::E),
    (0x46, "F", Key::F),
    (0x47, "G", Key::G),
    (0x48, "H", Key::H),
    (0x49, "I", Key::I),
    (0x4A, "J", Key::J),
    (0x4B, "K", Key::K),
    (0x4C, "L", Key::L),
    (0x4D, "M", Key::M),
    (0x4E, "N", Key::N),
    (0x4F, "O", Key::O),
    (0x50, "P", Key::P),
    (0x51, "Q", Key::Q),
    (0x52, "R", Key::R),
    (0x53, "S", Key::S),
    (0x54, "T", Key::T),
    (0x55, "U", Key::U),
    (0x56, "V", Key::V),
    (0x57, "W", Key::W),
    (0x58, "X", Key::X),
    (0x59, "Y", Key::Y),
    (0x5A, "Z", Key::Z),
    (0x70, "F1", Key::F1),
    (0x71, "F2", Key::F2),
    (0x72, "F3", Key::F3),
    (0x73, "F4", Key::F4),
    (0x74, "F5", Key::F5),
    (0x75, "F6", Key::F6),
    (0x76, "F7", Key::F7),
    (0x77, "F8", Key::F8),
    (0x78, "F9", Key::F9),
    (0x79, "F10", Key::F10),
    (0x7A, "F11", Key::F11),
    (0x7B, "F12", Key::F12),
    (0x7C, "F13", Key::F13),
    (0x7D, "F14", Key::F14),
    (0x7E, "F15", Key::F15),
    (0x7F, "F16", Key::F16),
    (0x80, "F17", Key::F17),
    (0x81, "F18", Key::F18),
    (0x82, "F19", Key::F19),
    (0x83, "F20", Key::F20),
];

pub const VK_F: u16 = 0x46;
pub const VK_F8: u16 = 0x77;
pub const VK_F9: u16 = 0x78;
pub const VK_F10: u16 = 0x79;
/// Reserved by Windows for the debugger: `RegisterHotKey` always rejects it.
pub const VK_F12: u16 = 0x7B;

pub fn can_be_hotkey(vk: u16) -> bool {
    vk != VK_F12
}

pub fn name(vk: u16) -> String {
    KEYS.iter()
        .find(|(code, ..)| *code == vk)
        .map(|(_, name, _)| name.to_string())
        .unwrap_or_else(|| format!("VK {vk:#04X}"))
}

pub fn from_egui(key: Key) -> Option<u16> {
    KEYS.iter()
        .find(|(.., k)| *k == key)
        .map(|(code, ..)| *code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        assert_eq!(from_egui(Key::F), Some(VK_F));
        assert_eq!(from_egui(Key::F8), Some(VK_F8));
        assert_eq!(name(VK_F12), "F12");
        assert_eq!(name(0xFF), "VK 0xFF");
    }

    #[test]
    fn codes_are_unique() {
        for (i, (a, ..)) in KEYS.iter().enumerate() {
            assert!(KEYS[i + 1..].iter().all(|(b, ..)| a != b));
        }
    }
}
