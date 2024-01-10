use crate::hexadecimal_sprites::{HEXADECIMAL_SPRITES, HEX_SIZE};
use std::cmp::min;
use std::fs::read;

pub const STARTING_MEMORY: u16 = 0x200;
const MEM_SIZE: usize = 4096;
const MAX_PROGRAM_SIZE: usize = MEM_SIZE - (STARTING_MEMORY as usize);

pub struct Memory {
    pub value: [u8; MEM_SIZE],
}

impl Memory {
    pub fn query_hex_location(digit: u8) -> Option<u8> {
        if digit < 16 {
            Some(digit * (HEX_SIZE as u8))
        } else {
            None
        }
    }

    fn load_hex(mem_arr: &mut [u8; MEM_SIZE]) {
        let hex_iter = (0..(HEXADECIMAL_SPRITES.len() * HEX_SIZE))
            .step_by(HEX_SIZE)
            .zip(HEXADECIMAL_SPRITES);

        for (mem_start, sprite) in hex_iter {
            for (si, byte) in sprite.iter().enumerate() {
                mem_arr[mem_start + si] = *byte;
            }
        }
    }

    /**
     * Reads two consecutive words (1 byte) at `idx` from memory and concatenates them to a 2 byte instruction.
     */
    pub fn read_instruction(&self, idx: u16) -> u16 {
        if let [upper, lower, ..] = &self.value[idx as usize..idx as usize + 2] {
            (*upper as u16) << 8 | *lower as u16
        } else {
            panic!("Memory out of bounds.")
        }
    }

    pub fn empty() -> Memory {
        let mut mem_arr = [0; MEM_SIZE];
        Memory::load_hex(&mut mem_arr);

        Memory { value: mem_arr }
    }
    pub fn new(file_path: &str) -> Result<Memory, String> {
        read(file_path)
            .map(|bytes| {
                let truncated_file = &bytes[0..min(MAX_PROGRAM_SIZE, bytes.len())];
                let mut mem_arr: [u8; MEM_SIZE] = [0; MEM_SIZE];
                // front load with hex sprites
                Memory::load_hex(&mut mem_arr);
                let mut i = STARTING_MEMORY as usize;
                for b in truncated_file {
                    mem_arr[i] = *b;
                    i += 1;
                }
                Memory { value: mem_arr }
            })
            .map_err(|err| err.to_string())
    }
}
