use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

// TODO Need to figure out a way to only redraw the display a maximum of 60 times per second
// TODO Need to only execute ~700 instructions per second

pub struct EmuDisplay {
    inner: widget::Widget,
    pixel_mat: Rc<RefCell<[[bool; 64]; 32]>>,
    key_pressed: Rc<RefCell<Option<u8>>>,
}

impl EmuDisplay {
    // our constructor
    pub fn new(label: &str) -> Self {
        let mut inner = widget::Widget::default()
            .with_size(64 * 5, 32 * 5)
            .with_label(label)
            .center_of_parent();
        inner.set_frame(enums::FrameType::FlatBox);

        let pixel_mat = [[false; 64]; 32];
        let pixel_mat = Rc::from(RefCell::from(pixel_mat));
        let draw_mat = pixel_mat.clone();
        let key_pressed = Rc::new(RefCell::new(None));
        let handle_key = key_pressed.clone();
        inner.draw(move |i| {
            let mat = draw_mat.borrow();
            for row in 0..31 {
                for col in 0..63 {
                    if mat[row as usize][col as usize] {
                        draw::draw_rect_fill(
                            i.x() + col * 5,
                            i.y() + row * 5,
                            5,
                            5,
                            enums::Color::White,
                        );
                    } else {
                        draw::draw_rect_fill(
                            i.x() + col * 5,
                            i.y() + row * 5,
                            5,
                            5,
                            enums::Color::Black,
                        );
                    }
                }
            }
        });
        inner.handle(move |i, ev| match ev {
            enums::Event::Push => {
                i.redraw();
                true
            }
            enums::Event::KeyDown => {
                let key = app::event_key();
                let key_char = key.to_char();
                let chip8_key = match key_char {
                    Some('1') => Some(0x1),
                    Some('2') => Some(0x2),
                    Some('3') => Some(0x3),
                    Some('4') => Some(0xC),
                    Some('q') => Some(0x4),
                    Some('w') => Some(0x5),
                    Some('e') => Some(0x6),
                    Some('r') => Some(0xD),
                    Some('a') => Some(0x7),
                    Some('s') => Some(0x8),
                    Some('d') => Some(0x9),
                    Some('f') => Some(0xE),
                    Some('z') => Some(0xA),
                    Some('x') => Some(0x0),
                    Some('c') => Some(0xB),
                    Some('v') => Some(0xF),
                    _ => None,
                };
                if let Some(k) = chip8_key {
                    *handle_key.borrow_mut() = Some(k);
                }
                true
            }
            enums::Event::KeyUp => {
                *handle_key.borrow_mut() = None;
                true
            }
            _ => false,
        });
        Self {
            inner,
            pixel_mat,
            key_pressed,
        }
    }

    pub fn get_key_pressed(&self) -> Option<u8> {
        *self.key_pressed.borrow()
    }
}

// Extend widget::Widget via the member `inner` and add other initializers and constructors
widget_extends!(EmuDisplay, widget::Widget, inner);
