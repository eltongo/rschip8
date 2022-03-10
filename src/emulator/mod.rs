mod cpu;
mod ui;
mod input;

pub type Chip8Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Chip8Result<T> = Result<T, Chip8Error>;

use std::{thread, time::Duration};
use sdl2::event::Event;
use cpu::CPU;
use ui::Screen;
use input::Keyboard;

pub fn run(file: &str) -> Chip8Result<()> {
    let sdl_context = sdl2::init()?;
    let title = format!("{} - {}", "rschip8", file);
    let mut screen = Screen::new(&sdl_context, &title)?;
    let mut kb = Keyboard::new();
    let mut cpu = CPU::from_file(file)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut timer_60hz = 0;
    'emulator: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'emulator;
                }
                Event::KeyDown { keycode: Some(code), .. } => {
                    kb.key_down(code);
                }
                Event::KeyUp { keycode: Some(code), .. } => {
                    kb.key_up(code);
                }
                _ => {}
            }
        }

        cpu.tick(&kb, &mut screen.display_buffer, timer_60hz == 0)?;
        if timer_60hz == 0 { screen.draw()?; }

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
        timer_60hz = (timer_60hz + 1) % 10;
    }

    Ok(())
}
