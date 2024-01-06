#![allow(dead_code)]
extern crate sdl2;

pub mod displays;
pub mod hexadecimal_sprites;
pub mod interpreters;
pub mod keypads;
pub mod memory;
pub mod timers;
pub mod waves;

use waves::Audio;

use crate::memory::Memory;
use crate::{displays::Display, interpreters::Interpreter};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let window = video_subsystem
        .window("Chip8", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let file_path = "foobar.txt";
    let mut program: Memory = Memory::new(file_path)?;

    // create Interpreter
    let display = Display::new(window)?;
    let audio = Audio::new(audio_subsystem)?;

    let mut interpreter = Interpreter::new(&mut program, display, audio);

    // pass pump to execute program
    let mut event_pump = sdl_context.event_pump()?;
    interpreter.execute_program(&mut event_pump, 30);

    Ok(())
}
