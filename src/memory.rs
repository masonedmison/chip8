use crate::hexadecimal_sprites::HEXADECIMAL_SPRITES;
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
            Some(digit)
        } else {
            None
        }
    }

    // TODO ensure we properly pad so hexadecimals are situated correctly in RAM
    fn load_hex(mem_arr: &mut [u8; MEM_SIZE]) {
        for (i, sprite) in HEXADECIMAL_SPRITES.iter().enumerate() {
            for (si, byte) in sprite.iter().enumerate() {
                mem_arr[i + si] = *byte;
            }
        }
    }

    /**
     * Reads two consecutive words (1 byte) at `idx` from memory and concatenates them to a 2 byte instruction.
     */
    pub fn read_instruction(&self, idx: u16) -> u16 {
        if let [upper, lower, ..] = &self.value[idx as usize..idx as usize + 2] {
            (*upper as u16) << 8 + *lower as u16
        } else {
            panic!("Memory out of bounds.")
        }
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
