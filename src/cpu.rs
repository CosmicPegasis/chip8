use crate::display::EmuDisplay;
use crate::keyboard;
use crate::ram::RAM;
use crate::register::Reg;
use crate::stack::Stack;
use fltk::prelude::*;
use rand;
use std::cmp;

pub struct CPU {
    reg: Reg,
    memory: RAM,
    stack: Stack,
    display: EmuDisplay,
    found_key: Option<u8>,
}

#[derive(Debug)]
struct Decoded {
    // For the u8 values only lower nibble is used
    // For u16 only lower 12 bits are used
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
    upper: u8,
}
// TODO Create a font set in memory
impl Decoded {
    pub fn new(opcode: u16) -> Self {
        Decoded {
            x: ((opcode & 0x0F00) >> 8) as u8,
            y: ((opcode & 0x00F0) >> 4) as u8,
            n: (opcode & 0x000F) as u8,
            nn: (opcode & 0x00FF) as u8,
            nnn: opcode & 0x0FFF,
            upper: ((opcode & 0xF000) >> 12) as u8,
        }
    }
}

impl CPU {
    pub fn default() -> Self {
        let display = EmuDisplay::new("Display");
        CPU::new(display)
    }
    pub fn new(display: EmuDisplay) -> Self {
        let mut cpu = CPU {
            reg: Reg::default(),
            memory: RAM::default(),
            stack: Stack::default(),
            display,
            found_key: None,
        };
        cpu.reg.pc = 0x200;
        cpu
    }
    fn setup_fonts(&mut self) {
        let fonts = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        for i in 0..fonts.len() {
            self.memory.write(0x50 + i, fonts[i]);
        }
    }

    pub fn load_rom(&mut self, rom: &str) {
        self.memory.load(rom).unwrap();
    }
    pub fn fetch(&mut self) -> u16 {
        // println!("PC: {:x}, Cycle: {}", self.reg.pc, self.cycle_count);
        let opcode: u8 = match self.memory.read(self.reg.pc as usize) {
            Ok(opcode) => opcode,
            Err(e) => panic!("Error fetching upper byte: {}", e),
        };
        let mut opcode: u16 = (opcode as u16) << 8;
        let lower = match self.memory.read(self.reg.pc as usize + 1) {
            Ok(lower) => lower,
            Err(e) => panic!("Error fetching lower byte: {}", e),
        };
        opcode = opcode | (lower as u16);
        opcode
    }
    pub fn run(&mut self) {
        // decode for chip-8
        // I have not implemented 0nnn instruction which was used on old chip-8 interpreters
        let opcode = self.fetch();
        self.reg.pc += 2;
        if opcode == 0x00E0 {
            self.clear_screen();
        } else if opcode == 0x00EE {
            self.return_from_subroutine();
        } else {
            let decoded = Decoded::new(opcode);
            // println!("{:?}", decoded);
            match decoded.upper {
                0x1 => self.jump_to_address(decoded),
                0x2 => self.call_subroutine(decoded),
                0x3 => self.skip_next_instruction_if_equal(decoded),
                0x4 => self.skip_next_instruction_if_not_equal(decoded),
                0x5 => self.skip_next_instruction_if_equal_register(decoded),
                0x6 => self.set_register(decoded),
                0x7 => self.add_to_register(decoded),
                0x8 => self.apply_op(decoded),
                0x9 => self.skip_next_instruction_if_not_equal_register(decoded),
                0xA => self.set_memory_addr(decoded),
                0xB => self.jump_to_address_with_offset(decoded),
                0xC => self.rnd_and(decoded),
                0xD => self.disp_sprite(decoded),
                0xE => self.skip_next_instruction_cond(decoded),
                0xF => self.misc_op(decoded),
                _ => panic!("Unknown opcode {:x}", opcode),
            }
        }
    }

    fn jump_to_address(&mut self, decoded: Decoded) {
        // println!("JMP {:x}", decoded.nnn);
        self.reg.pc = decoded.nnn;
    }

    fn call_subroutine(&mut self, decoded: Decoded) {
        println!("CALL {:x}", decoded.nnn);
        self.reg.sp += 1;
        self.stack.push(self.reg.pc);
        self.reg.pc = decoded.nnn;
    }

    fn return_from_subroutine(&mut self) {
        println!("RET");
        self.reg.pc = self.stack.pop().unwrap();
        self.reg.sp -= 1;
    }

    fn skip_next_instruction_if_equal(&mut self, decoded: Decoded) {
        println!("SE V{:x} {:x}", decoded.x, decoded.nn);
        if self.reg.v[decoded.x as usize] == decoded.nn {
            self.reg.pc += 2;
        }
    }

    fn skip_next_instruction_if_not_equal(&mut self, decoded: Decoded) {
        println!("SNE V{:x} {:x}", decoded.x, decoded.nn);
        if self.reg.v[decoded.x as usize] != decoded.nn {
            self.reg.pc += 2;
        }
    }

    fn skip_next_instruction_if_equal_register(&mut self, decoded: Decoded) {
        println!("SE V{:x} V{:x}", decoded.x, decoded.y);
        if decoded.n != 0 {
            panic!("Unknown Opcode: skip next with non zero lowest nibble (n)");
        }
        if self.reg.v[decoded.x as usize] == self.reg.v[decoded.y as usize] {
            self.reg.pc += 2;
        }
    }
    fn skip_next_instruction_if_not_equal_register(&mut self, decoded: Decoded) {
        println!("SNE V{:x} V{:x}", decoded.x, decoded.y);
        if decoded.n != 0 {
            panic!("Unknown Opcode: skip next with non zero lowest nibble (n)");
        }
        if self.reg.v[decoded.x as usize] != self.reg.v[decoded.y as usize] {
            self.reg.pc += 2;
        }
    }

    fn set_register(&mut self, decoded: Decoded) {
        println!("SET V{:x} {:x}", decoded.x, decoded.nn);
        self.reg.v[decoded.x as usize] = decoded.nn;
    }

    fn add_to_register(&mut self, decoded: Decoded) {
        // Am I supposed to check for overflow?
        println!("ADD V{:x} {:x}", decoded.x, decoded.nn);
        let a = self.reg.v[decoded.x as usize] as u16;
        let b = decoded.nn as u16;
        if (a + b) > 255 {
            self.reg.v[decoded.x as usize] = ((a + b) % 256) as u8;
        } else {
            self.reg.v[decoded.x as usize] += decoded.nn;
        }
    }

    fn apply_op(&mut self, decoded: Decoded) {
        match decoded.n {
            0x0 => self.copy_registers(decoded),
            0x1 => self.or_registers(decoded),
            0x2 => self.and_registers(decoded),
            0x3 => self.xor_registers(decoded),
            0x4 => self.add_registers(decoded),
            0x5 => self.sub_registers(decoded),
            0x6 => self.shift_registers_right(decoded),
            0x7 => self.subn_registers(decoded),
            0xE => self.shift_registers_left(decoded),
            _ => panic!("Unknown Operation: {:x}", decoded.n),
        }
    }

    fn copy_registers(&mut self, decoded: Decoded) {
        println!("COPY V{:x} V{:x}", decoded.x, decoded.y);
        self.reg.v[decoded.x as usize] = self.reg.v[decoded.y as usize];
    }

    fn or_registers(&mut self, decoded: Decoded) {
        println!("OR V{:x} V{:x}", decoded.x, decoded.y);
        self.reg.v[decoded.x as usize] =
            self.reg.v[decoded.x as usize] | self.reg.v[decoded.y as usize];
        // quirk
        // self.reg.v[0xF] = 0;
    }

    fn and_registers(&mut self, decoded: Decoded) {
        println!("AND V{:x} V{:x}", decoded.x, decoded.y);
        self.reg.v[decoded.x as usize] =
            self.reg.v[decoded.x as usize] & self.reg.v[decoded.y as usize];
        // quirk
        // self.reg.v[0xF] = 0;
    }

    fn xor_registers(&mut self, decoded: Decoded) {
        println!("XOR V{:x} V{:x}", decoded.x, decoded.y);
        self.reg.v[decoded.x as usize] =
            self.reg.v[decoded.x as usize] ^ self.reg.v[decoded.y as usize];
        // quirk
        // self.reg.v[0xF] = 0;
    }

    fn add_registers(&mut self, decoded: Decoded) {
        println!("ADD V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.reg.v[decoded.x as usize] as u16;
        let b = self.reg.v[decoded.y as usize] as u16;
        self.reg.v[decoded.x as usize] = (a + b) as u8;
        if (a + b) > 255 {
            self.reg.v[0xF] = 1;
        } else {
            self.reg.v[0xF] = 0;
        }
    }

    fn sub_registers(&mut self, decoded: Decoded) {
        println!("SUB V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.reg.v[decoded.x as usize];
        let b = self.reg.v[decoded.y as usize];
        if a > b {
            self.reg.v[decoded.x as usize] = a - b;
            self.reg.v[0xF] = 1;
        } else {
            self.reg.v[decoded.x as usize] = ((256 + a as u16) - b as u16) as u8;
            self.reg.v[0xF] = 0;
        }
    }

    fn shift_registers_right(&mut self, decoded: Decoded) {
        println!("SHR V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.reg.v[decoded.x as usize];
        self.reg.v[decoded.x as usize] = self.reg.v[decoded.x as usize] >> 1;
        if a & 0x1 == 1 {
            self.reg.v[0xF] = 1;
        } else {
            self.reg.v[0xF] = 0;
        }
    }

    fn shift_registers_left(&mut self, decoded: Decoded) {
        println!("SHL V{:x}", decoded.x);
        let a = self.reg.v[decoded.x as usize];
        self.reg.v[decoded.x as usize] = self.reg.v[decoded.x as usize] << 1;
        if a & 0x80 == 0x80 {
            self.reg.v[0xF] = 1;
        } else {
            self.reg.v[0xF] = 0;
        }
    }

    fn subn_registers(&mut self, decoded: Decoded) {
        println!("SUBN V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.reg.v[decoded.x as usize];
        let b = self.reg.v[decoded.y as usize];
        if b > a {
            self.reg.v[decoded.x as usize] = b - a;
            self.reg.v[0xF] = 1;
        } else {
            self.reg.v[decoded.x as usize] = ((256 + b as u16) - a as u16) as u8;
            self.reg.v[0xF] = 0;
        }
    }

    fn set_memory_addr(&mut self, decoded: Decoded) {
        println!("SET I {:x}", decoded.nnn);
        self.reg.i = decoded.nnn;
    }

    fn jump_to_address_with_offset(&mut self, decoded: Decoded) {
        println!("JP V{:x} {:x}", 0, decoded.nnn);
        self.reg.pc = decoded.nnn + (self.reg.v[0] as u16);
    }

    fn rnd_and(&mut self, decoded: Decoded) {
        println!("RND V{:x} {:x}", decoded.x, decoded.nn);
        self.reg.v[decoded.x as usize] = rand::random::<u8>() & decoded.nn;
    }

    fn misc_op(&mut self, decoded: Decoded) {
        match decoded.nn {
            0x07 => self.ld_delay_timer(decoded),
            0x0A => self.ld_register_key(decoded),
            0x15 => self.ld_delay_timer_register(decoded),
            0x18 => self.ld_sound_timer_register(decoded),
            0x1E => self.add_i_register(decoded),
            0x29 => self.ld_font_char(decoded),
            0x33 => self.ld_bcd_register(decoded),
            0x55 => self.sv_registers_to_mem(decoded),
            0x65 => self.ld_registers_from_mem(decoded),
            _ => panic!("Unknown misc op {:x}", decoded.nn),
        }
    }

    fn ld_delay_timer(&mut self, decoded: Decoded) {
        println!("LD V{:x} DT", decoded.x);
        self.reg.v[decoded.x as usize] = self.reg.delay_timer;
    }

    fn ld_delay_timer_register(&mut self, decoded: Decoded) {
        // TODO Implement decrementing of delay timer
        println!("LD DT V{:x}", decoded.x);
        self.reg.delay_timer = self.reg.v[decoded.x as usize];
    }

    fn ld_sound_timer_register(&mut self, decoded: Decoded) {
        println!("LD ST V{:x}", decoded.x);
        self.reg.sound_time = self.reg.v[decoded.x as usize];
    }

    fn add_i_register(&mut self, decoded: Decoded) {
        // handle overflow?
        println!("ADD I V{:x}", decoded.x);
        self.reg.i += self.reg.v[decoded.x as usize] as u16;
    }

    fn ld_bcd_register(&mut self, decoded: Decoded) {
        println!("LD BCD V{:x}", decoded.x);
        let num = self.reg.v[decoded.x as usize];
        self.memory.write(self.reg.i as usize, (num / 100) as u8);
        self.memory
            .write(self.reg.i as usize + 1, ((num % 100) / 10) as u8);
        self.memory.write(self.reg.i as usize + 2, (num % 10) as u8);
    }

    fn sv_registers_to_mem(&mut self, decoded: Decoded) {
        println!("LD [I] V{:x}", decoded.x);
        for i in 0..decoded.x + 1 {
            self.memory
                .write(self.reg.i as usize + i as usize, self.reg.v[i as usize]);
        }
        // // quirk
        // self.reg.i += decoded.x as u16 + 1;
    }

    fn ld_registers_from_mem(&mut self, decoded: Decoded) {
        println!("LD V{:x} [I]", decoded.x);
        for i in 0..decoded.x + 1 {
            self.reg.v[i as usize] = self.memory.read(self.reg.i as usize + i as usize).unwrap();
        }
        // quirk
        // self.reg.i += decoded.x as u16 + 1;
    }

    fn clear_screen(&mut self) {
        println!("CLS");
        {
            let mut mat = self.display.pixel_mat.borrow_mut();
            for i in 0..32 {
                for j in 0..64 {
                    mat[i][j] = false;
                }
            }
        }

        // self.display.redraw();
    }
    fn disp_sprite(&mut self, decoded: Decoded) {
        let mut collision = false;
        let start_x = self.reg.v[decoded.x as usize] % 64;
        let start_y = self.reg.v[decoded.y as usize] % 32;
        {
            let mut mat = self.display.pixel_mat.borrow_mut();
            for y in start_y..cmp::min(start_y + decoded.n, 32) {
                let mut mask = self
                    .memory
                    .read(self.reg.i as usize + y as usize - start_y as usize)
                    .unwrap();
                let mut rev_mask = 0;
                for i in 0..8 {
                    rev_mask = (rev_mask << 1) | (mask & 0x1);
                    mask = mask >> 1;
                }
                mask = rev_mask;
                for x in start_x..cmp::min(start_x + 8, 64) {
                    let pixel = mat[y as usize][x as usize];
                    if pixel {
                        collision = true;
                    }
                    mat[y as usize][x as usize] = pixel ^ ((mask & 0x1) != 0);
                    mask = mask >> 1;
                }
            }
        }
        if collision {
            self.reg.v[0xF] = 1;
        } else {
            self.reg.v[0xF] = 0;
        }
        //self.display.redraw();
    }

    fn ld_font_char(&mut self, decoded: Decoded) {
        println!("LD F V{:x}", decoded.x);
        self.reg.i = self.reg.v[decoded.x as usize] as u16 * 5 + 0x50;
    }

    fn ld_register_key(&mut self, decoded: Decoded) {
        println!("LD V{:x} K", decoded.x);
        match self.found_key {
            Some(k) => {
                if let Some(k) = *self.display.last_key_up.borrow() {
                    self.reg.v[decoded.x as usize] = k;
                    self.found_key = None;
                } else {
                    self.reg.pc -= 2;
                }
            }
            None => {
                if let Some(k) = *self.display.last_key_down.borrow() {
                    self.found_key = Some(k);
                }
                self.reg.pc -= 2;
            }
        }
    }

    fn skip_next_instruction_cond(&mut self, decoded: Decoded) {
        match decoded.nn {
            0x9E => self.skip_next_instruction_if_key_pressed(decoded),
            0xA1 => self.skip_next_instruction_if_key_not_pressed(decoded),
            _ => panic!("Unknown skip cond {:x}", decoded.nn),
        }
    }

    fn skip_next_instruction_if_key_pressed(&mut self, decoded: Decoded) {
        println!("SKP V{:x}", decoded.x);
        if self.display.keys_pressed.borrow()[self.reg.v[decoded.x as usize] as usize] {
            self.reg.pc += 2;
        }
    }

    fn skip_next_instruction_if_key_not_pressed(&mut self, decoded: Decoded) {
        println!("SKNP V{:x}", decoded.x);
        if !self.display.keys_pressed.borrow()[self.reg.v[decoded.x as usize] as usize] {
            self.reg.pc += 2;
        }
    }
    pub fn update_timers(&mut self) {
        if self.reg.delay_timer > 0 {
            self.reg.delay_timer -= 1;
        }
        if self.reg.sound_time > 0 {
            self.reg.sound_time -= 1;
        }
    }
    pub fn should_beep(&self) -> bool {
        self.reg.sound_time > 0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fetch() {
        let mut cpu = CPU::default();
        cpu.memory.cart_size = 2 + 0x200;
        cpu.memory.cart[0x200] = 0x12;
        cpu.memory.cart[0x201] = 0x34;
        assert_eq!(cpu.fetch(), 0x1234);
    }

    #[test]
    fn test_decode() {
        let decoded = Decoded::new(0x1234);
        assert_eq!(decoded.x, 0x2);
        assert_eq!(decoded.y, 0x3);
        assert_eq!(decoded.n, 0x4);
        assert_eq!(decoded.nn, 0x34);
        assert_eq!(decoded.nnn, 0x234);
        assert_eq!(decoded.upper, 0x1);
    }

    #[test]
    fn test_jump_to_address() {
        let mut cpu = CPU::default();
        cpu.jump_to_address(Decoded::new(0x1234));
        assert_eq!(cpu.reg.pc, 0x0234);
    }
    #[test]
    fn test_skip_next_instruction_if_equal() {
        let mut cpu = CPU::default();
        cpu.reg.v[0xe] = 0x12;
        cpu.skip_next_instruction_if_equal(Decoded::new(0x3e12));
        assert_eq!(cpu.reg.pc, 0x202);
    }
    #[test]
    fn test_skip_next_instruction_if_not_equal() {
        let mut cpu = CPU::default();
        cpu.reg.v[0xe] = 0x12;
        cpu.skip_next_instruction_if_not_equal(Decoded::new(0x4e13));
        assert_eq!(cpu.reg.pc, 0x202);
    }
    #[test]
    fn test_apply_op() {
        let mut cpu = CPU::default();

        // Test 0x8xy0 (LD Vx, Vy)
        cpu.reg.v[0] = 0;
        cpu.reg.v[1] = 0x42;
        cpu.apply_op(Decoded::new(0x8010));
        assert_eq!(cpu.reg.v[0], 0x42);

        // Test 0x8xy1 (OR Vx, Vy)
        cpu.reg.v[0] = 0b1010;
        cpu.reg.v[1] = 0b0101;
        cpu.apply_op(Decoded::new(0x8011));
        assert_eq!(cpu.reg.v[0], 0b1111);

        // Test 0x8xy2 (AND Vx, Vy)
        cpu.reg.v[0] = 0b1010;
        cpu.reg.v[1] = 0b0110;
        cpu.apply_op(Decoded::new(0x8012));
        assert_eq!(cpu.reg.v[0], 0b0010);

        // Test 0x8xy3 (XOR Vx, Vy)
        cpu.reg.v[0] = 0b1010;
        cpu.reg.v[1] = 0b0110;
        cpu.apply_op(Decoded::new(0x8013));
        assert_eq!(cpu.reg.v[0], 0b1100);

        // Test 0x8xy4 (ADD Vx, Vy)
        cpu.reg.v[0] = 200;
        cpu.reg.v[1] = 100;
        cpu.apply_op(Decoded::new(0x8014));
        assert_eq!(cpu.reg.v[0], 44); // 300 % 256 = 44
        assert_eq!(cpu.reg.v[0xF], 1); // Carry

        cpu.reg.v[0] = 50;
        cpu.reg.v[1] = 50;
        cpu.apply_op(Decoded::new(0x8014));
        assert_eq!(cpu.reg.v[0], 100);
        assert_eq!(cpu.reg.v[0xF], 0); // No carry

        // Test 0x8xy5 (SUB Vx, Vy)
        cpu.reg.v[0] = 10;
        cpu.reg.v[1] = 5;
        cpu.apply_op(Decoded::new(0x8015));
        assert_eq!(cpu.reg.v[0], 5);
        assert_eq!(cpu.reg.v[0xF], 1); // No borrow

        cpu.reg.v[0] = 5;
        cpu.reg.v[1] = 10;
        cpu.apply_op(Decoded::new(0x8015));
        assert_eq!(cpu.reg.v[0], 251); // 256 - 5 = 251
        assert_eq!(cpu.reg.v[0xF], 0); // Borrow

        // Test 0x8xy6 (SHR Vx {, Vy})
        cpu.reg.v[0] = 0b1011;
        cpu.apply_op(Decoded::new(0x8006));
        assert_eq!(cpu.reg.v[0], 0b0101);
        assert_eq!(cpu.reg.v[0xF], 1);

        cpu.reg.v[0] = 0b1010;
        cpu.apply_op(Decoded::new(0x8006));
        assert_eq!(cpu.reg.v[0], 0b0101);
        assert_eq!(cpu.reg.v[0xF], 0);

        // Test 0x8xy7 (SUBN Vx, Vy)
        cpu.reg.v[0] = 5;
        cpu.reg.v[1] = 10;
        cpu.apply_op(Decoded::new(0x8017));
        assert_eq!(cpu.reg.v[0], 5);
        assert_eq!(cpu.reg.v[0xF], 1); // No borrow

        cpu.reg.v[0] = 10;
        cpu.reg.v[1] = 5;
        cpu.apply_op(Decoded::new(0x8017));
        assert_eq!(cpu.reg.v[0], 251); // 256 - 5 = 251
        assert_eq!(cpu.reg.v[0xF], 0); // Borrow

        // Test 0x8xyE (SHL Vx {, Vy})
        cpu.reg.v[0] = 0b01000000;
        cpu.apply_op(Decoded::new(0x800E));
        assert_eq!(cpu.reg.v[0], 0b10000000);
        assert_eq!(cpu.reg.v[0xF], 0);

        cpu.reg.v[0] = 0b10000000;
        cpu.apply_op(Decoded::new(0x800E));
        assert_eq!(cpu.reg.v[0], 0);
        assert_eq!(cpu.reg.v[0xF], 1);
    }

    #[test]
    fn test_disp_sprite() {
        let mut cpu = CPU::default();

        // Set up sprite data in memory
        cpu.reg.i = 0x300;
        cpu.memory.cart_size = 0x303;
        cpu.memory.cart[0x300] = 0b11110000;
        cpu.memory.cart[0x301] = 0b10010000;
        cpu.memory.cart[0x302] = 0b11110000;

        // Test 1: Display sprite at (0, 0)
        cpu.reg.v[0] = 0;
        cpu.reg.v[1] = 0;
        cpu.disp_sprite(Decoded::new(0xD013)); // Display 3-byte sprite at (V0, V1)

        {
            let mat = cpu.display.pixel_mat.borrow();
            assert_eq!(
                mat[0][0..8],
                [true, true, true, true, false, false, false, false]
            );
            assert_eq!(
                mat[1][0..8],
                [true, false, false, true, false, false, false, false]
            );
            assert_eq!(
                mat[2][0..8],
                [true, true, true, true, false, false, false, false]
            );
        }
        assert_eq!(cpu.reg.v[0xF], 0); // No collision

        // Test 2: Display sprite at (2, 1) - partial overlap
        cpu.reg.v[0] = 2;
        cpu.reg.v[1] = 1;
        cpu.disp_sprite(Decoded::new(0xD013));

        {
            let mat = cpu.display.pixel_mat.borrow();
            assert_eq!(
                mat[0][0..8],
                [true, true, true, true, false, false, false, false]
            );
            assert_eq!(
                mat[1][0..8],
                [true, false, true, false, true, true, false, false]
            );
            assert_eq!(
                mat[2][0..8],
                [true, true, false, true, false, true, false, false]
            );
            assert_eq!(
                mat[3][0..8],
                [false, false, true, true, true, true, false, false]
            );
        }
        assert_eq!(cpu.reg.v[0xF], 1); // Collision detected

        // Test 3: Display sprite at edge of screen (wrap around)
        cpu.clear_screen();
        cpu.reg.v[0] = 63;
        cpu.reg.v[1] = 31;
        cpu.disp_sprite(Decoded::new(0xD013));

        {
            let mat = cpu.display.pixel_mat.borrow();
            assert_eq!(mat[31][63], true);
            assert_eq!(mat[0][0], false);
            assert_eq!(mat[1][0], false);
        }
    }
}
