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
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
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
