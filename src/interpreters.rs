use std::time::Duration;

use rand::Rng;

extern crate sdl2;

use crate::displays::Drawable;
use crate::input_driver::EventPollable;
use crate::keypads::Keypad;
use crate::memory::{Memory, STARTING_MEMORY};
use crate::timers::TimerActions;

const OPCODE_SIZE: u16 = 2;

pub struct Interpreter<D, I, ST, DT>
where
    I: EventPollable,
    D: Drawable,
    ST: TimerActions,
    DT: TimerActions,
{
    memory: Memory,
    registers: Registers,
    stack: Stack,
    display: D,
    keypad: [bool; 16],
    input_driver: I,
    sound_timer: ST,
    delay_timer: DT,
}

impl<D, I, ST, DT> Interpreter<D, I, ST, DT>
where
    I: EventPollable,
    D: Drawable,
    ST: TimerActions,
    DT: TimerActions,
{
    pub fn new(
        memory: Memory,
        display: D,
        input_driver: I,
        sound_timer: ST,
        delay_timer: DT,
    ) -> Interpreter<D, I, ST, DT> {
        let stack = [0; 16];
        let registers = Registers::new();
        let keypad = [false; 16];
        Interpreter {
            memory,
            registers,
            stack,
            display,
            keypad,
            input_driver,
            sound_timer,
            delay_timer,
        }
    }

    pub fn execute_program(&mut self) {
        let frame_sleep = Duration::from_millis(2);
        while let Ok(Keypad { keypad, .. }) = self.input_driver.poll() {
            std::thread::sleep(frame_sleep);

            self.keypad = keypad;


            let opcode = self.memory.read_instruction(self.registers.pc);
            match self.execute_opcode(opcode as u16) {
                ProgramCounter::Next => self.registers.pc += OPCODE_SIZE,
                ProgramCounter::Skip(true) => self.registers.pc += 2 * OPCODE_SIZE,
                ProgramCounter::Skip(false) => self.registers.pc += OPCODE_SIZE,
                ProgramCounter::Jump(addr) => self.registers.pc = addr as u16,
            }

            self.delay_timer.decrement();
            self.sound_timer.decrement();



        }
    }

    fn loop_until_keypressed(&mut self) -> Option<u8> {
        let result;
        'until: loop {
            std::thread::sleep(Duration::from_millis(10));

            match self.input_driver.poll() {
                Ok(Keypad {
                    keypad,
                    last_pressed: lp @ Some(_),
                }) => {
                    result = lp;
                    self.keypad = keypad;
                    break 'until;
                }
                Err(_) => panic!("interuppted"),
                _ => ()
            }
        }
        result
    }

    fn execute_opcode(&mut self, raw_opcode: u16) -> ProgramCounter {
        let opcode = Opcodes::from_bytes(raw_opcode).expect("Expected a valid opcode.");
        match opcode {
            Opcodes::CLS => {
                self.display.clear();
                ProgramCounter::Next
            }
            Opcodes::RET => {
                self.registers.sp -= 1;
                ProgramCounter::Jump(self.stack[self.registers.sp] as usize)
            }
            Opcodes::JP(Addr(value)) => ProgramCounter::Jump(value as usize),
            Opcodes::CALL(Addr(value)) => {
                self.stack[self.registers.sp] = self.registers.pc + OPCODE_SIZE;
                self.registers.sp += 1;
                ProgramCounter::Jump(value as usize)
            }
            Opcodes::SEByte(RegisterN(x), Byte(value)) => {
                ProgramCounter::Skip(self.registers.v[x] == value)
            }
            Opcodes::SNE(RegisterN(x), Byte(value)) => {
                ProgramCounter::Skip(self.registers.v[x] != value)
            }
            Opcodes::SEReg(RegisterN(x), RegisterN(y)) => {
                ProgramCounter::Skip(self.registers.v[x] == self.registers.v[y])
            }
            Opcodes::LDByte(RegisterN(x), Byte(value)) => {
                self.registers.v[x] = value;
                ProgramCounter::Next
            }
            Opcodes::ADDByte(RegisterN(x), Byte(kk)) => {
                let vx = self.registers.v[x] as u16;
                let val = kk as u16;

                self.registers.v[x] = (vx + val) as u8;
                ProgramCounter::Next
            }
            Opcodes::LDReg(RegisterN(x), RegisterN(y)) => {
                self.registers.v[x] = self.registers.v[y];
                ProgramCounter::Next
            }
            Opcodes::OR(RegisterN(x), RegisterN(y)) => {
                self.registers.v[x] |= self.registers.v[y];
                ProgramCounter::Next
            }
            Opcodes::ANDReg(RegisterN(x), RegisterN(y)) => {
                self.registers.v[x] &= self.registers.v[y];
                ProgramCounter::Next
            }
            Opcodes::XOR(RegisterN(x), RegisterN(y)) => {
                self.registers.v[x] ^= self.registers.v[y];
                ProgramCounter::Next
            }
            Opcodes::ADDReg(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.v[x];
                let vy = self.registers.v[y];
                let (z, overflow) = vx.overflowing_add(vy);
                if overflow {
                    self.registers.set_vf(1)
                } else {
                    self.registers.set_vf(0)
                }
                self.registers.v[x] = z;
                ProgramCounter::Next
            }
            Opcodes::SUB(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.v[x];
                let vy = self.registers.v[y];
                if vx > vy {
                    self.registers.set_vf(1)
                } else {
                    self.registers.set_vf(0)
                }
                self.registers.v[x] = vx.wrapping_sub(vy);
                ProgramCounter::Next
            }
            Opcodes::SHR(RegisterN(x)) => {
                let vx = self.registers.v[x];
                self.registers.set_vf(vx & 1);
                self.registers.v[x] /= 2;
                ProgramCounter::Next
            }
            Opcodes::SUBN(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.v[x];
                let vy = self.registers.v[y];
                if vy > vx {
                    self.registers.set_vf(1)
                } else {
                    self.registers.set_vf(0)
                }
                self.registers.v[x] = vy.wrapping_sub(vx);
                ProgramCounter::Next
            }
            Opcodes::SHL(RegisterN(x)) => {
                let vx = self.registers.v[x];
                if vx >= 128 {
                    self.registers.set_vf(1)
                } else {
                    self.registers.set_vf(0)
                }
                self.registers.v[x] *= 2;
                ProgramCounter::Next
            }
            Opcodes::SNEReg(RegisterN(x), RegisterN(y)) => {
                let vx = self.registers.v[x];
                let vy = self.registers.v[y];
                ProgramCounter::Skip(vx != vy)
            }
            Opcodes::LDI(Addr(v)) => {
                self.registers.i = v;
                ProgramCounter::Next
            }
            Opcodes::JPV0(Addr(v)) => {
                let addr = v + self.registers.v[0] as u16;
                ProgramCounter::Jump(addr as usize)
            }
            Opcodes::RND(RegisterN(x), Byte(kk)) => {
                let mut rng = rand::thread_rng();
                self.registers.v[x] = rng.gen::<u8>() & kk;
                ProgramCounter::Next
            }
            Opcodes::DRW(RegisterN(x), RegisterN(y), Nibble(n)) => {
                let addr = self.registers.i as usize;
                let sprite = &self.memory.value[addr..addr + n as usize];
                let start_coord = (self.registers.v[x] as usize, self.registers.v[y] as usize);
                let collided = self.display.draw_at(sprite, start_coord);
                self.registers.set_vf(if collided { 1 } else { 0 });
                ProgramCounter::Next
            }
            Opcodes::SKP(RegisterN(x)) => {
                ProgramCounter::Skip(self.keypad[self.registers.v[x] as usize])
            }
            Opcodes::SKNP(RegisterN(x)) => {
                ProgramCounter::Skip(!self.keypad[self.registers.v[x] as usize])
            }
            Opcodes::LDVXWITHDT(RegisterN(x)) => {
                self.registers.v[x] = self.delay_timer.get();
                ProgramCounter::Next
            }
            Opcodes::LDK(RegisterN(x)) => {
                if let Some(keycode) = self.loop_until_keypressed() {
                    self.registers.v[x] = keycode;
                }
                ProgramCounter::Next
            }
            Opcodes::LDDTWITHVX(RegisterN(x)) => {
                self.delay_timer.set(self.registers.v[x]);
                ProgramCounter::Next
            }
            Opcodes::LDST(RegisterN(x)) => {
                self.sound_timer.set(self.registers.v[x]);
                ProgramCounter::Next
            }
            Opcodes::ADDI(RegisterN(x)) => {
                self.registers.i += self.registers.v[x] as u16;
                self.registers.set_vf(if self.registers.i > 0x0F00 { 1 } else { 0 });
                ProgramCounter::Next
            }
            Opcodes::LDSPRITE(RegisterN(x)) => {
                if let Some(mem_loc) = Memory::query_hex_location(self.registers.v[x]) {
                    self.registers.i = mem_loc as u16;
                }
                ProgramCounter::Next
            }
            Opcodes::LDBCD(RegisterN(x)) => {
                let get_digit = |n: u32, d: u32| (n / u32::pow(10, d) % 10);
                let vx = self.registers.v[x] as u32;
                let mem_loc = self.registers.i as usize;
                self.memory.value[mem_loc] = get_digit(vx, 2) as u8;
                self.memory.value[mem_loc + 1] = get_digit(vx, 1) as u8;
                self.memory.value[mem_loc + 2] = get_digit(vx, 0) as u8;
                ProgramCounter::Next
            }
            Opcodes::LDTHROUGH(RegisterN(x)) => {
                let mem_loc_start = self.registers.i;
                self.registers.v[0..=x as usize]
                    .iter()
                    .zip(mem_loc_start..)
                    .for_each(|(value, mem_loc)| {
                        self.memory.value[mem_loc as usize] = *value;
                    });
                ProgramCounter::Next
            }
            Opcodes::LDTHROUGHINTOI(RegisterN(x)) => {
                let indices = (0..=x).zip(self.registers.i..);
                for (n, i) in indices {
                    let mem_cell = self.memory.value[i as usize];
                    self.registers.v[n as usize] = mem_cell;
                }
                ProgramCounter::Next
            }
        }
    }
}

struct Registers {
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: usize,
}

impl Registers {
    fn new() -> Registers {
        // TODO not sure that these are "good" init values for registers
        Registers {
            // registers V0-VF
            v: [0; 16],
            i: STARTING_MEMORY,
            pc: STARTING_MEMORY,
            sp: 0,
        }
    }

    fn set_vf(&mut self, value: u8) {
        self.v[15] = value;
    }
}

#[derive(PartialEq, Debug)]
enum ProgramCounter {
    Next,
    Skip(bool),
    Jump(usize),
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
        match destructure_raw_opcode(raw_opcode) {
            (0, _, _, 0) => Some(Opcodes::CLS),
            (0, _, _, 0xE) => Some(Opcodes::RET),
            (1, _, _, _) => Some(Opcodes::JP(get_addr(raw_opcode))),
            (2, _, _, _) => Some(Opcodes::CALL(get_addr(raw_opcode))),
            (3, x, _, _) => Some(Opcodes::SEByte(RegisterN(x), byte(raw_opcode))),
            (4, x, _, _) => Some(Opcodes::SNE(RegisterN(x), byte(raw_opcode))),
            (5, x, y, 0) => Some(Opcodes::SEReg(RegisterN(x), RegisterN(y))),
            (6, x, _, _) => Some(Opcodes::LDByte(RegisterN(x), byte(raw_opcode))),
            (7, x, _, _) => Some(Opcodes::ADDByte(RegisterN(x), byte(raw_opcode))),
            (8, x, y, 0) => Some(Opcodes::LDReg(RegisterN(x), RegisterN(y))),
            (8, x, y, 1) => Some(Opcodes::OR(RegisterN(x), RegisterN(y))),
            (8, x, y, 2) => Some(Opcodes::ANDReg(RegisterN(x), RegisterN(y))),
            (8, x, y, 3) => Some(Opcodes::XOR(RegisterN(x), RegisterN(y))),
            (8, x, y, 4) => Some(Opcodes::ADDReg(RegisterN(x), RegisterN(y))),
            (8, x, y, 5) => Some(Opcodes::SUB(RegisterN(x), RegisterN(y))),
            (8, x, _, 6) => Some(Opcodes::SHR(RegisterN(x))),
            (8, x, y, 7) => Some(Opcodes::SUBN(RegisterN(x), RegisterN(y))),
            (8, x, _, 0xE) => Some(Opcodes::SHL(RegisterN(x))),
            (9, x, y, 0) => Some(Opcodes::SNEReg(RegisterN(x), RegisterN(y))),
            (0xA, _, _, _) => Some(Opcodes::LDI(get_addr(raw_opcode))),
            (0xB, _, _, _) => Some(Opcodes::JPV0(get_addr(raw_opcode))),
            (0xC, x, _, _) => Some(Opcodes::RND(RegisterN(x), byte(raw_opcode))),
            (0xD, x, y, n) => Some(Opcodes::DRW(RegisterN(x), RegisterN(y), Nibble(n))),
            (0xE, x, 9, 0xE) => Some(Opcodes::SKP(RegisterN(x))),
            (0xE, x, 0xA, 1) => Some(Opcodes::SKNP(RegisterN(x))),
            (0xF, x, 0, 7) => Some(Opcodes::LDVXWITHDT(RegisterN(x))),
            (0xF, x, 0, 0xA) => Some(Opcodes::LDK(RegisterN(x))),
            (0xF, x, 1, 5) => Some(Opcodes::LDDTWITHVX(RegisterN(x))),
            (0xF, x, 1, 8) => Some(Opcodes::LDST(RegisterN(x))),
            (0xF, x, 1, 0xE) => Some(Opcodes::ADDI(RegisterN(x))),
            (0xF, x, 2, 9) => Some(Opcodes::LDSPRITE(RegisterN(x))),
            (0xF, x, 3, 3) => Some(Opcodes::LDBCD(RegisterN(x))),
            (0xF, x, 5, 5) => Some(Opcodes::LDTHROUGH(RegisterN(x))),
            (0xF, x, 6, 5) => Some(Opcodes::LDTHROUGHINTOI(RegisterN(x))),
            _ => None,
        }
    }
}

fn get_addr(raw_opcode: u16) -> Addr {
    Addr(raw_opcode & 0x0FFF)
}

fn byte(raw_opcode: u16) -> Byte {
    Byte((raw_opcode & 0x00FF) as u8)
}

fn destructure_raw_opcode(raw_opcode: u16) -> (u16, usize, usize, u8) {
    (
        raw_opcode >> 12,
        ((raw_opcode & 0x0F00) >> 8) as usize,
        ((raw_opcode & 0x00F0) >> 4) as usize,
        (raw_opcode & 0x000F) as u8,
    )
}

struct Addr(u16);
struct RegisterN(usize);
struct Nibble(u8);
struct Byte(u8);

#[cfg(test)]
#[path = "./interpreters_test.rs"]
mod intpreters_test;
