use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;
pub struct EmuDisplay {
    inner: widget::Widget,
    pixel_mat: Rc<RefCell<[[bool; 64]; 32]>>,
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
        //       let handle_mat = pixel_mat.clone();
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
            _ => false,
        });
        Self { inner, pixel_mat }
    }
}

// Extend widget::Widget via the member `inner` and add other initializers and constructors
widget_extends!(EmuDisplay, widget::Widget, inner);
