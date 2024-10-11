use crate::keyboard::map_modern_to_chip8;
use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

// TODO Need to figure out a way to only redraw the display a maximum of 60 times per second
// TODO Need to only execute ~700 instructions per second

pub struct EmuDisplay {
    pub inner: widget::Widget,
    pub pixel_mat: Rc<RefCell<[[bool; 64]; 32]>>,
    pub keys_pressed: Rc<RefCell<[bool; 16]>>,
    pub last_key_down: Rc<RefCell<Option<u8>>>,
    pub last_key_up: Rc<RefCell<Option<u8>>>,
}

impl EmuDisplay {
    pub fn new(label: &str) -> Self {
        let mut inner = widget::Widget::default()
            .with_size(64 * 10, 32 * 10)
            .with_label(label)
            .center_of_parent();
        inner.set_frame(enums::FrameType::NoBox);

        let pixel_mat = [[false; 64]; 32];
        let pixel_mat = Rc::from(RefCell::from(pixel_mat));
        let draw_mat = pixel_mat.clone();
        let keys_pressed = Rc::new(RefCell::new([false; 16]));
        let handle_keys_pressed = keys_pressed.clone();
        let last_key_down = Rc::new(RefCell::new(None));
        let last_key_up = Rc::new(RefCell::new(None));
        let last_key_down_clone = last_key_down.clone();
        let last_key_up_clone = last_key_up.clone();
        inner.draw(move |i| {
            let mat = draw_mat.borrow();
            for row in 0..32 {
                for col in 0..64 {
                    if mat[row as usize][col as usize] {
                        draw::draw_rect_fill(
                            i.x() + col * 10,
                            i.y() + row * 10,
                            10,
                            10,
                            enums::Color::White,
                        );
                    } else {
                        draw::draw_rect_fill(
                            i.x() + col * 10,
                            i.y() + row * 10,
                            10,
                            10,
                            enums::Color::Black,
                        );
                    }
                }
            }
        });
        inner.handle(move |i, ev| match ev {
            enums::Event::KeyDown => {
                let key = app::event_key();
                let key_char = key.to_char();
                let chip8_key = match key_char {
                    Some(x) => map_modern_to_chip8(x),
                    _ => None,
                };
                if let Some(k) = chip8_key {
                    handle_keys_pressed.borrow_mut()[k as usize] = true;
                    *last_key_down_clone.borrow_mut() = Some(k);
                    println!("Key pressed: {:x}", k);
                }
                *last_key_up_clone.borrow_mut() = None;
                true
            }
            enums::Event::KeyUp => {
                let key = app::event_key();
                let key_char = key.to_char();
                let chip8_key = match key_char {
                    Some(x) => map_modern_to_chip8(x),
                    _ => None,
                };
                if let Some(k) = chip8_key {
                    handle_keys_pressed.borrow_mut()[k as usize] = false;
                    println!("Key released: {:x}", k);
                    *last_key_up_clone.borrow_mut() = Some(k);
                }
                *last_key_down_clone.borrow_mut() = None;
                true
            }
            enums::Event::Shortcut => {
                let key_char = app::event_key().to_char();
                let chip8_key = match key_char {
                    Some(x) => map_modern_to_chip8(x),
                    _ => None,
                };
                if let Some(k) = chip8_key {
                    handle_keys_pressed.borrow_mut()[k as usize] = true;
                    println!("Key pressed: {:x}", k);
                    *last_key_down_clone.borrow_mut() = Some(k);
                }
                *last_key_up_clone.borrow_mut() = None;
                true
            }
            _ => {
                *last_key_up_clone.borrow_mut() = None;
                *last_key_down_clone.borrow_mut() = None;
                false
            }
        });
        Self {
            inner,
            pixel_mat,
            keys_pressed,
            last_key_down,
            last_key_up,
        }
    }
}

// Extend widget::Widget via the member `inner` and add other initializers and constructors
widget_extends!(EmuDisplay, widget::Widget, inner);
