extern crate sdl2;

use sdl2::keyboard::Keycode;

const VALID_KEYS: [Keycode; 16] = [
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::F,
    Keycode::H,
    Keycode::J,
    Keycode::K,
    Keycode::L,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::R,
    Keycode::U,
    Keycode::I,
    Keycode::O,
    Keycode::P,
];

pub struct KeyPad {
    pub pressed_down_key: Option<Keycode>,
}

impl KeyPad {
    pub fn new() -> KeyPad {
        KeyPad {
            pressed_down_key: None,
        }
    }
    fn is_valid_key(key: &Keycode) -> bool {
        if VALID_KEYS.contains(key) {
            true
        } else {
            false
        }
    }

    pub fn from_i32(value: i32) -> Option<Keycode> {
        Keycode::from_i32(value).and_then(|k| {
            if KeyPad::is_valid_key(&k) {
                Some(k)
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, key: &Keycode) -> bool {
        if KeyPad::is_valid_key(key) {
            self.pressed_down_key = Some(*key);
            true
        } else {
            false
        }
    }

    pub fn as_u8(&self) -> Result<u16, String> {
        self.pressed_down_key
            .ok_or("last_pressed key is None.".to_string())
            .map(|k| k as u16)
    }
}
