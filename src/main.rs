extern crate sdl2;

pub mod displays;
pub mod hexadecimal_sprites;
pub mod input_driver;
pub mod interpreters;
pub mod keypads;
pub mod memory;
pub mod timers;
pub mod waves;

use input_driver::InputDriver;
use timers::{DelayTimer, SoundTimer};
use waves::Audio;

use crate::memory::Memory;
use crate::{displays::Display, interpreters::Interpreter};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let file_path = "games/MAZE";
    let program: Memory = Memory::new(file_path)?;

    // create Interpreter
    let display = Display::new(video_subsystem)?;
    let audio = Audio::new(audio_subsystem)?;
    let input = InputDriver::new(&sdl_context);
    let delay_timer = DelayTimer { value: 0 };
    let sound_timer = SoundTimer::new(audio);

    let mut interpreter = Interpreter::new(program, display, input, sound_timer, delay_timer);

    interpreter.execute_program(60);

    Ok(())
}
