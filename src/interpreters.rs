use std::time::Duration;

use rand::Rng;
use sdl2::event::Event;

extern crate sdl2;

use crate::displays::Display;
use crate::keypads::KeyPad;
use crate::memory::{Memory, STARTING_MEMORY};
use crate::timers::{TimerActions, Timers};
use crate::waves::Audio;
use sdl2::EventPump;

pub struct Interpreter<'a> {
    memory: &'a mut Memory,
    registers: Registers,
    stack: Stack,
    display: Display,
    keypad: KeyPad,
    timers: Timers,
}

impl<'a> Interpreter<'a> {
    pub fn new(memory: &'a mut Memory, display: Display, audio: Audio) -> Interpreter<'a> {
        let stack = [0; 16];
        let registers = Registers::new();
        let keypad = KeyPad::new();
        let timers = Timers::new(audio);
        Interpreter {
            memory,
            registers,
            stack,
            display,
            keypad,
            timers,
        }
    }

    // TODO account for keys from event pump
    pub fn execute_program(&mut self, pump: &mut EventPump, fps: u32) {
        let mut frame_ctr = 0;
        let frame_sleep = Duration::from_secs(1) / fps;
        'program: loop {
            std::thread::sleep(frame_sleep);

            if frame_ctr % 30 == 0 {
                let opcode = self.memory.read_instruction(self.registers.pc);
                self.execute_opcode(opcode, pump)
            }

            // decr timers
            if (frame_ctr % 60) == 0 {
                self.timers.sound.decrement();
                self.timers.delay.decrement();

                frame_ctr = 0;
            }

            frame_ctr += 1;
        }
    }

    fn loop_until_keypressed(keypad: &mut KeyPad, pump: &mut EventPump) -> Option<u16> {
        let result;
        'until: loop {
            std::thread::sleep(Duration::from_millis(10));

            for event in pump.poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(kc), ..
                    } => {
                        if keypad.set(&kc) {
                            result = keypad.as_u8().ok();
                            break 'until;
                        }
                    }
                    _ => (),
                }
            }
        }
        result
    }

    fn execute_opcode(&mut self, raw_opcode: u16, pump: &mut EventPump) {
        let opcode = Opcodes::from_bytes(raw_opcode).expect("Expected a valid opcode.");
        match opcode {
            Opcodes::CLS => self.display.clear_display(),
            Opcodes::RET => {
                self.registers.pc = self.stack[self.registers.sc as usize];
                self.registers.sc -= 1;
            }
            Opcodes::JP(Addr(value)) => self.registers.pc = value,
            Opcodes::CALL(Addr(value)) => {
                self.registers.sc += 1;
                self.stack[self.registers.sc as usize] = self.registers.pc;
                self.registers.pc = value;
            }
            Opcodes::SEByte(RegisterN(x), Byte(value)) => {
                if self.registers.accessible[x as usize] == value {
                    self.registers.pc += 2;
                }
            }
            Opcodes::SNE(RegisterN(x), Byte(value)) => {
                if self.registers.accessible[x as usize] != value {
                    self.registers.pc += 2;
                }
            }
            Opcodes::SEReg(RegisterN(x), RegisterN(y)) => {
                if self.registers.accessible[x as usize] == self.registers.accessible[y as usize] {
                    self.registers.pc += 2;
                }
            }
            Opcodes::LDByte(RegisterN(x), Byte(value)) => {
                self.registers.accessible[x as usize] = value;
            }
            Opcodes::ADDByte(RegisterN(x), Byte(value)) => {
                self.registers.accessible[x as usize] += value;
            }
            Opcodes::LDReg(RegisterN(x), RegisterN(y)) => {
                self.registers.accessible[x as usize] = self.registers.accessible[y as usize];
            }
            Opcodes::OR(RegisterN(x), RegisterN(y)) => {
                let vy = self.registers.accessible[y as usize];
                self.registers.accessible[x as usize] |= vy;
            }
            Opcodes::ANDReg(RegisterN(x), RegisterN(y)) => {
                let vy = self.registers.accessible[y as usize];
                self.registers.accessible[x as usize] &= vy;
            }
            Opcodes::XOR(RegisterN(x), RegisterN(y)) => {
                let vy = self.registers.accessible[y as usize];
                self.registers.accessible[x as usize] ^= vy;
            }
            Opcodes::ADDReg(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.accessible[x as usize];
                let vy = self.registers.accessible[y as usize];
                // TODO is it correct to use wrapping here?
                let (z, overflow) = vx.overflowing_add(vy);
                if overflow {
                    self.registers.f = 1;
                } else {
                    self.registers.f = 0;
                }
                self.registers.accessible[x as usize] = z;
            }
            Opcodes::SUB(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.accessible[x as usize];
                let vy = self.registers.accessible[y as usize];
                if vx > vy {
                    self.registers.f = 1;
                } else {
                    self.registers.f = 0;
                }
                self.registers.accessible[x as usize] = vx - vy;
            }
            Opcodes::SHR(RegisterN(x)) => {
                let vx = self.registers.accessible[x as usize];
                if 1 & vx == 1 {
                    self.registers.f = 1
                } else {
                    self.registers.f = 0;
                }
                self.registers.accessible[x as usize] /= 2;
            }
            Opcodes::SUBN(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.accessible[x as usize];
                let vy = self.registers.accessible[y as usize];
                if vy > vx {
                    self.registers.f = 1;
                } else {
                    self.registers.f = 0;
                }
                self.registers.accessible[x as usize] = vy - vx;
            }
            Opcodes::SHL(RegisterN(x)) => {
                let vx = self.registers.accessible[x as usize];
                if vx >= 128 {
                    self.registers.f = 1;
                } else {
                    self.registers.f = 0;
                }
                self.registers.accessible[x as usize] *= 2;
            }
            Opcodes::SNEReg(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.accessible[x as usize];
                let vy = self.registers.accessible[y as usize];
                if vx != vy {
                    self.registers.pc += 2;
                }
            }
            Opcodes::LDI(Addr(v)) => {
                self.registers.i = v;
            }
            Opcodes::JPV0(Addr(v)) => {
                self.registers.pc = v + self.registers.accessible[0] as u16;
            }
            Opcodes::RND(RegisterN(x), Byte(kk)) => {
                let a = rand::thread_rng().gen_range(0..=255) & kk;
                self.registers.accessible[x as usize] = a;
            }
            Opcodes::DRW(RegisterN(x), RegisterN(y), Nibble(n)) => {
                let addr = self.registers.i as usize;
                let sprite = &self.memory.value[addr..addr + n as usize];
                let collided = self.display.draw_at(sprite, (x as usize, y as usize));
                self.registers.f = if collided { 1 } else { 0 }
            }
            Opcodes::SKP(RegisterN(x)) => {
                if let Ok(keycode) = self.keypad.as_u8() {
                    if keycode == self.registers.accessible[x as usize] as u16 {
                        self.registers.pc += 2
                    }
                }
            }
            Opcodes::SKNP(RegisterN(x)) => {
                if let Ok(keycode) = self.keypad.as_u8() {
                    if keycode != self.registers.accessible[x as usize] as u16 {
                        self.registers.pc += 2
                    }
                }
            }
            Opcodes::LDVXWITHDT(RegisterN(x)) => self.timers.delay.set(x),
            Opcodes::LDK(RegisterN(x)) => {
                if let Some(keycode) = Interpreter::loop_until_keypressed(&mut self.keypad, pump) {
                    // TODO seems weird that we are assuming keycode will fit into a byte here...
                    self.registers.accessible[x as usize] = keycode as u8;
                }
            }
            Opcodes::LDDTWITHVX(RegisterN(x)) => {
                self.timers.delay.set(self.registers.accessible[x as usize]);
            }
            Opcodes::LDST(RegisterN(x)) => {
                self.timers.sound.set(self.registers.accessible[x as usize]);
            }
            Opcodes::ADDI(RegisterN(x)) => {
                self.registers.i += self.registers.accessible[x as usize] as u16;
            }
            Opcodes::LDSPRITE(RegisterN(x)) => {
                if let Some(mem_loc) = Memory::query_hex_location(x) {
                    self.registers.i = mem_loc as u16;
                }
            }
            Opcodes::LDBCD(RegisterN(x)) => {
                let get_digit = |n, d| n / (u8::pow(10, d) % 10);
                let vx = self.registers.accessible[x as usize];
                let mem_loc = self.registers.i as usize;
                self.memory.value[mem_loc] = get_digit(vx, 3);
                self.memory.value[mem_loc + 1] = get_digit(vx, 2);
                self.memory.value[mem_loc + 2] = get_digit(vx, 1);
            }
            Opcodes::LDTHROUGH(RegisterN(x)) => {
                let mem_loc_start = self.registers.i;
                self.registers.accessible[0..x as usize]
                    .iter()
                    .zip(mem_loc_start..)
                    .for_each(|(value, mem_loc)| {
                        self.memory.value[mem_loc as usize] = *value;
                    })
            }
            Opcodes::LDTHROUGHINTOI(RegisterN(x)) => {
                let indices = (0..x).zip(self.registers.i..);
                for (n, i) in indices {
                    let mem_cell = self.memory.value[i as usize];
                    self.registers.accessible[n as usize] = mem_cell;
                }
            }
        }
    }
}

struct Registers {
    accessible: [u8; 15],
    f: u8,
    i: u16,
    pc: u16,
    sc: u8,
}

impl Registers {
    fn new() -> Registers {
        // TODO not sure that these are "good" init values for registers
        Registers {
            // registers V0-VE
            accessible: [0; 15],
            f: 0,
            i: STARTING_MEMORY,
            pc: STARTING_MEMORY,
            sc: 0,
        }
    }
}

type Stack = [u16; 16];

enum Opcodes {
    CLS,
    RET,
    JP(Addr),
    CALL(Addr),
    SEByte(RegisterN, Byte),
    SNE(RegisterN, Byte),
    SEReg(RegisterN, RegisterN),
    LDByte(RegisterN, Byte),
    ADDByte(RegisterN, Byte),
    LDReg(RegisterN, RegisterN),
    OR(RegisterN, RegisterN),
    ANDReg(RegisterN, RegisterN),
    XOR(RegisterN, RegisterN),
    ADDReg(RegisterN, RegisterN),
    SUB(RegisterN, RegisterN),
    SHR(RegisterN),
    SUBN(RegisterN, RegisterN),
    SHL(RegisterN),
    SNEReg(RegisterN, RegisterN),
    LDI(Addr),
    JPV0(Addr),
    RND(RegisterN, Byte),
    DRW(RegisterN, RegisterN, Nibble),
    SKP(RegisterN),
    SKNP(RegisterN),
    LDVXWITHDT(RegisterN),
    LDK(RegisterN),
    LDDTWITHVX(RegisterN),
    LDST(RegisterN),
    ADDI(RegisterN),
    LDSPRITE(RegisterN),
    LDBCD(RegisterN),
    LDTHROUGH(RegisterN),
    LDTHROUGHINTOI(RegisterN),
}

impl Opcodes {
    fn from_bytes(raw_opcode: u16) -> Option<Opcodes> {
        // we match on first and last hexadecimal digit
        match destructure_raw_opcode(raw_opcode) {
            (0, _, 0) => Some(Opcodes::CLS),
            (0, _, 0xE) => Some(Opcodes::RET),
            (1, _, _) => Some(Opcodes::JP(get_addr(raw_opcode))),
            (2, _, _) => Some(Opcodes::CALL(get_addr(raw_opcode))),
            (3, _, _) => {
                let (reg_x, byte) = reg_and_byte(raw_opcode);
                Some(Opcodes::SEByte(reg_x, byte))
            }
            (4, _, _) => {
                let (reg_x, byte) = reg_and_byte(raw_opcode);
                Some(Opcodes::SNE(reg_x, byte))
            }
            (5, y, 0) => Some(Opcodes::SEReg(regx(raw_opcode), RegisterN(y))),
            (6, _, _) => {
                let (reg_x, byte) = reg_and_byte(raw_opcode);
                Some(Opcodes::LDByte(reg_x, byte))
            }
            (7, _, _) => {
                let (reg_x, byte) = reg_and_byte(raw_opcode);
                Some(Opcodes::ADDByte(reg_x, byte))
            }
            (8, y, 0) => Some(Opcodes::LDReg(regx(raw_opcode), RegisterN(y))),
            (8, y, 1) => Some(Opcodes::OR(regx(raw_opcode), RegisterN(y))),
            (8, y, 2) => Some(Opcodes::ANDReg(regx(raw_opcode), RegisterN(y))),
            (8, y, 3) => Some(Opcodes::XOR(regx(raw_opcode), RegisterN(y))),
            (8, y, 4) => Some(Opcodes::ADDReg(regx(raw_opcode), RegisterN(y))),
            (8, y, 5) => Some(Opcodes::SUB(regx(raw_opcode), RegisterN(y))),
            (8, _, 6) => Some(Opcodes::SHR(regx(raw_opcode))),
            (8, y, 7) => Some(Opcodes::SUBN(regx(raw_opcode), RegisterN(y))),
            (8, _, 0xE) => Some(Opcodes::SHL(regx(raw_opcode))),
            (9, y, 0) => Some(Opcodes::SNEReg(regx(raw_opcode), RegisterN(y))),
            (0xA, _, _) => Some(Opcodes::LDI(get_addr(raw_opcode))),
            (0xB, _, _) => Some(Opcodes::JPV0(get_addr(raw_opcode))),
            (0xC, _, _) => {
                let (reg_x, byte) = reg_and_byte(raw_opcode);
                Some(Opcodes::RND(reg_x, byte))
            }
            (0xD, y, n) => Some(Opcodes::DRW(regx(raw_opcode), RegisterN(y), Nibble(n))),
            (0xE, 9, 0xE) => Some(Opcodes::SKP(regx(raw_opcode))),
            (0xE, 0xA, 1) => Some(Opcodes::SKNP(regx(raw_opcode))),
            (0xF, 0, 7) => Some(Opcodes::LDVXWITHDT(regx(raw_opcode))),
            (0xF, 0, 0xA) => Some(Opcodes::LDK(regx(raw_opcode))),
            (0xF, 1, 5) => Some(Opcodes::LDDTWITHVX(regx(raw_opcode))),
            (0xF, 1, 8) => Some(Opcodes::LDST(regx(raw_opcode))),
            (0xF, 1, 0xE) => Some(Opcodes::ADDI(regx(raw_opcode))),
            (0xF, 2, 9) => Some(Opcodes::LDSPRITE(regx(raw_opcode))),
            (0xF, 3, 3) => Some(Opcodes::LDSPRITE(regx(raw_opcode))),
            (0xF, 5, 5) => Some(Opcodes::LDTHROUGH(regx(raw_opcode))),
            (0xF, 6, 5) => Some(Opcodes::LDTHROUGH(regx(raw_opcode))),
            _ => None,
        }
    }
}

fn get_addr(raw_opcode: u16) -> Addr {
    Addr(raw_opcode & 0x0FFF)
}

fn regx(raw_opcode: u16) -> RegisterN {
    RegisterN(((raw_opcode & 0x0F00) >> 8) as u8)
}

fn byte(raw_opcode: u16) -> Byte {
    Byte((raw_opcode & 0x00FF) as u8)
}

fn reg_and_byte(raw_opcode: u16) -> (RegisterN, Byte) {
    (regx(raw_opcode), byte(raw_opcode))
}

fn destructure_raw_opcode(raw_opcode: u16) -> (u16, u8, u8) {
    (
        raw_opcode >> 12,
        ((raw_opcode & 0x00F0) >> 4) as u8,
        (raw_opcode & 0x000F) as u8,
    )
}

struct Addr(u16);
struct RegisterN(u8);
struct Nibble(u8);
struct Byte(u8);

mod tests {
    use crate::interpreters::{byte, destructure_raw_opcode, get_addr, regx};
    #[test]
    fn get_addr_works() {
        let opcode = 0x2111;
        let actual = get_addr(opcode);
        let expected = 0x111;

        assert_eq!(actual.0, expected)
    }
    #[test]
    fn regx_works() {
        let opcode = 0x3133;
        let actual = regx(opcode);
        let expected = 1;
        assert_eq!(actual.0, expected)
    }
    #[test]
    fn byte_works() {
        let opcode = 0x2133;
        let actual = byte(opcode);
        let expected = 0x33;

        assert_eq!(actual.0, expected)
    }

    #[test]
    fn destructure_raw_opcode_works() {
        let x = 0xF0B9;
        let result_x = destructure_raw_opcode(x);
        assert_eq!(result_x, (0xF, 0xB, 9));

        let y = 0x902E;
        let result_y = destructure_raw_opcode(y);
        assert_eq!(result_y, (0x9, 2, 0xE))
    }
}
