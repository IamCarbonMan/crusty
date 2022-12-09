use std::{
    env, fs,
    time::Duration
};

mod cpu;
mod gpu;
mod font;
mod audio;
mod input;

use crate::audio::get_audio;
use crate::cpu::Cpu;
use crate::gpu::Gpu;
use crate::input::Input;

fn main() -> Result<(), String> {
    let ctx = sdl2::init()?;
    let mut gpu = Gpu::new(&ctx, 10);
    let audio = get_audio(&ctx);
    let mut input = Input::new(&ctx);
    let args: Vec<String> = env::args().collect();
    let rom = fs::read(&args[1]).unwrap();
    let mut cpu = Cpu::new();
    cpu.load(&rom);
    while let keys = input.poll() {
       let state = cpu.tick(keys);
        if !state.screen.is_none() {
            gpu.draw(&state.screen.unwrap());
        }
        if state.sound {
            audio.resume();
        } else {
            audio.pause();
        }
        ::std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}