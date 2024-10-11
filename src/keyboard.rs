use std::collections::HashMap;
use std::io::{self, Write};

pub fn map_modern_to_chip8(modern_key: char) -> Option<u8> {
    match modern_key {
        '1' => Some(0x1),
        '2' => Some(0x2),
        '3' => Some(0x3),
        '4' => Some(0xC),
        'q' => Some(0x4),
        'w' => Some(0x5),
        'e' => Some(0x6),
        'r' => Some(0xD),
        'a' => Some(0x7),
        's' => Some(0x8),
        'd' => Some(0x9),
        'f' => Some(0xE),
        'z' => Some(0xA),
        'x' => Some(0x0),
        'c' => Some(0xB),
        'v' => Some(0xF),
        _ => None,
    }
}
