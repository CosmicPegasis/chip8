#[allow(unused)]
mod cpu;
mod display;
mod keyboard;
mod ram;
mod register;
mod stack;
use cpu::CPU;
use display::EmuDisplay;
use fltk::{prelude::*, *};
use rodio::{source::SineWave, source::Source, Decoder, OutputStream};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    let my_app = app::App::default().with_scheme(app::Scheme::Gleam);
    let mut wind = window::Window::new(100, 100, 640, 320, "Chip-8 Emu");
    let display = EmuDisplay::new("Display");
    wind.end();
    wind.show();

    let cpu = Rc::new(RefCell::new(CPU::new(display)));
    cpu.borrow_mut().load_rom("roms/glitchGhost.ch8");
    let cpu_clone = cpu.clone();
    // run approximately 700 cycle per second

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let screen_update_callback = move |handle| {
        wind.redraw();
        cpu_clone.borrow_mut().update_timers();
        if cpu_clone.borrow().should_beep() {
            let source = SineWave::new(440.0).take_duration(Duration::from_secs_f32(5.0 / 60.0));
            stream_handle.play_raw(source.convert_samples()).unwrap();
        }
        app::repeat_timeout3(1.0 / 60.0, handle);
    };
    let run_cpu_callback = move |handle| {
        cpu.borrow_mut().run();
        app::repeat_timeout3(1.0 / 720.0, handle);
    };
    app::add_timeout3(1.0 / 60.0, screen_update_callback);
    app::add_timeout3(1.0 / 720.0, run_cpu_callback);
    my_app.run().unwrap();
}
