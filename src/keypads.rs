extern crate sdl2;

pub struct Keypad {
    pub keypad: [bool; 16],
    pub last_pressed: Option<u8>,
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            keypad: [false; 16],
            last_pressed: None,
        }
    }
}
