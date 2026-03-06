use eframe::egui;

pub fn terminal_key_bytes(key: egui::Key, modifiers: egui::Modifiers) -> Option<Vec<u8>> {
    use egui::Key::*;
    if modifiers.ctrl && !modifiers.shift && !modifiers.alt {
        let b: u8 = match key {
            A => 0x01,
            B => 0x02,
            C => 0x03,
            D => 0x04,
            E => 0x05,
            F => 0x06,
            G => 0x07,
            H => 0x08,
            I => 0x09,
            J => 0x0a,
            K => 0x0b,
            L => 0x0c,
            M => 0x0d,
            N => 0x0e,
            O => 0x0f,
            P => 0x10,
            Q => 0x11,
            R => 0x12,
            S => 0x13,
            T => 0x14,
            U => 0x15,
            V => 0x16,
            W => 0x17,
            X => 0x18,
            Y => 0x19,
            Z => 0x1a,
            _ => return None,
        };
        return Some(vec![b]);
    }
    if modifiers.is_none() {
        return match key {
            Enter => Some(vec![0x0d]),
            Backspace => Some(vec![0x7f]),
            Escape => Some(vec![0x1b]),
            Tab => Some(vec![0x09]),
            Delete => Some(b"\x1b[3~".to_vec()),
            Insert => Some(b"\x1b[2~".to_vec()),
            Home => Some(b"\x1b[H".to_vec()),
            End => Some(b"\x1b[F".to_vec()),
            PageUp => Some(b"\x1b[5~".to_vec()),
            PageDown => Some(b"\x1b[6~".to_vec()),
            ArrowUp => Some(b"\x1b[A".to_vec()),
            ArrowDown => Some(b"\x1b[B".to_vec()),
            ArrowLeft => Some(b"\x1b[D".to_vec()),
            ArrowRight => Some(b"\x1b[C".to_vec()),
            _ => None,
        };
    }
    if modifiers == egui::Modifiers::SHIFT && key == Tab {
        return Some(b"\x1b[Z".to_vec());
    }
    None
}
