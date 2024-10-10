mod display;
mod keyboard;
mod ram;
mod register;
mod stack;
use display::EmuDisplay;
use fltk::{prelude::*, *};

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gleam);
    app::background(255, 255, 255); // make the background white
    let mut wind = window::Window::new(100, 100, 400, 300, "Chip-8 Emu");
    let _ = EmuDisplay::new("Display");
    wind.end();
    wind.show();

    app.run().unwrap();
}
