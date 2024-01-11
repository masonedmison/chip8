use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use crate::keypads::Keypad;

pub trait EventPollable {
    fn poll(&mut self) -> Result<Keypad, ()>;
}

pub struct InputDriver {
    event_pump: EventPump,
}

impl InputDriver {
    pub fn new(sdl: &sdl2::Sdl) -> InputDriver {
        InputDriver {
            event_pump: sdl.event_pump().unwrap(),
        }
    }
}

impl EventPollable for InputDriver {
    fn poll(&mut self) -> Result<Keypad, ()> {
        for event in self.event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Err(());
            }
        }

        let keys: Vec<Keycode> = self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut chip8_keys = [false; 16];
        let mut last_pressed: Option<u8> = None;
        for key in keys {
            let index: Option<usize> = match key {
                Keycode::Q => Some(0x1),
                Keycode::W => Some(0x2),
                Keycode::E => Some(0x3),
                Keycode::R => Some(0xc),
                Keycode::A => Some(0x4),
                Keycode::S => Some(0x5),
                Keycode::D => Some(0x6),
                Keycode::F => Some(0xd),
                Keycode::U => Some(0x7),
                Keycode::I => Some(0x8),
                Keycode::O => Some(0x9),
                Keycode::P => Some(0xe),
                Keycode::J => Some(0xa),
                Keycode::K => Some(0x0),
                Keycode::L => Some(0xb),
                Keycode::Semicolon => Some(0xf),
                _ => None,
            };
            if let Some(chip_idx) = index {
                last_pressed = Some(chip_idx as u8);
                chip8_keys[chip_idx] = true;
            }
        }
        Ok(Keypad {
            keypad: chip8_keys,
            last_pressed: last_pressed,
        })
    }
}
