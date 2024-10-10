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

fn main() {
    let my_app = app::App::default().with_scheme(app::Scheme::Gleam);
    let mut wind = window::Window::new(100, 100, 400, 300, "Chip-8 Emu");
    let display = EmuDisplay::new("Display");
    wind.end();
    wind.show();

    //  app.run().unwrap();
    let mut cpu = CPU::new(display);
    cpu.load_rom("roms/IBMlogo.ch8");
    // run approximately 700 cycle per second
    let screen_update_callback = move |handle| {
        wind.redraw();
        app::repeat_timeout3(1.0 / 60.0, handle);
    };
    let run_cpu_callback = move |handle| {
        cpu.run();
        app::repeat_timeout3(1.0 / 700.0, handle);
    };
    app::add_timeout3(1.0 / 60.0, screen_update_callback);
    app::add_timeout3(1.0 / 700.0, run_cpu_callback);
    my_app.run().unwrap();
}
