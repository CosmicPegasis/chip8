use crate::ram::RAM;
use crate::register::Reg;
use crate::stack::Stack;
use rand;
pub struct CPU {
    registers: Reg,
    memory: RAM,
    stack: Stack,
    sp: u8,
    pc: u16,
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
        CPU {
            registers: Reg::default(),
            memory: RAM::default(),
            stack: Stack::default(),
            sp: 0,
            pc: 0x200,
        }
    }
    pub fn load_rom(&mut self, rom: &str) {
        self.memory.load(rom).unwrap();
    }
    pub fn fetch(&mut self) -> u16 {
        let opcode: u8 = match self.memory.read(self.pc as usize) {
            Ok(opcode) => opcode,
            Err(e) => panic!("Error fetching upper byte: {}", e),
        };
        let mut opcode: u16 = (opcode as u16) << 8;
        let lower = match self.memory.read(self.pc as usize + 1) {
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
        self.pc += 2;
        if opcode == 0x00E0 {
            self.clear_screen();
        } else if opcode == 0x00EE {
            self.return_from_subroutine();
        } else {
            let decoded = Decoded::new(opcode);
            println!("{:?}", decoded);
            match decoded.upper {
                0x1 => self.jump_to_address(decoded),
                0x2 => self.call_subroutine(decoded),
                0x3 => self.skip_next_instruction_if_equal(decoded),
                0x4 => self.skip_next_instruction_if_not_equal(decoded),
                0x5 => self.skip_next_instruction_if_equal_register(decoded),
                0x6 => self.set_register(decoded),
                0x7 => self.add_to_register(decoded),
                0x8 => self.apply_op(decoded),
                //                0x9 => self.skip_next_instruction_if_not_equal_register(decoded),
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
        println!("JMP {:x}", decoded.nnn);
        self.pc = decoded.nnn;
    }
    fn skip_next_instruction_if_equal(&mut self, decoded: Decoded) {
        println!("SE V{:x} {:x}", decoded.x, decoded.nn);
        if self.registers.v[decoded.x as usize] == decoded.nn {
            self.pc += 2;
        }
    }
    fn skip_next_instruction_if_not_equal(&mut self, decoded: Decoded) {
        println!("SNE V{:x} {:x}", decoded.x, decoded.nn);
        if self.registers.v[decoded.x as usize] != decoded.nn {
            self.pc += 2;
        }
    }
    fn skip_next_instruction_if_equal_register(&mut self, decoded: Decoded) {
        println!("SE V{:x} V{:x}", decoded.x, decoded.y);
        if decoded.n != 0 {
            panic!("Unknown Opcode: skip next with non zero lowest nibble (n)");
        }
        if self.registers.v[decoded.x as usize] == self.registers.v[decoded.y as usize] {
            self.pc += 2;
        }
    }
    fn set_register(&mut self, decoded: Decoded) {
        println!("SET V{:x} {:x}", decoded.x, decoded.nn);
        self.registers.v[decoded.x as usize] = decoded.nn;
    }
    fn add_to_register(&mut self, decoded: Decoded) {
        // Am I supposed to check for overflow?
        println!("ADD V{:x} {:x}", decoded.x, decoded.nn);
        self.registers.v[decoded.x as usize] += decoded.nn;
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
        self.registers.v[decoded.x as usize] = self.registers.v[decoded.y as usize];
    }
    fn or_registers(&mut self, decoded: Decoded) {
        println!("OR V{:x} V{:x}", decoded.x, decoded.y);
        self.registers.v[decoded.x as usize] =
            self.registers.v[decoded.x as usize] | self.registers.v[decoded.y as usize];
    }
    fn and_registers(&mut self, decoded: Decoded) {
        println!("AND V{:x} V{:x}", decoded.x, decoded.y);
        self.registers.v[decoded.x as usize] =
            self.registers.v[decoded.x as usize] & self.registers.v[decoded.y as usize];
    }
    fn xor_registers(&mut self, decoded: Decoded) {
        println!("XOR V{:x} V{:x}", decoded.x, decoded.y);
        self.registers.v[decoded.x as usize] =
            self.registers.v[decoded.x as usize] ^ self.registers.v[decoded.y as usize];
    }
    fn add_registers(&mut self, decoded: Decoded) {
        println!("ADD V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.registers.v[decoded.x as usize] as u16;
        let b = self.registers.v[decoded.y as usize] as u16;
        if (a + b) > 255 {
            self.registers.v[0xF] = 1;
        } else {
            self.registers.v[0xF] = 0;
        }
        self.registers.v[decoded.x as usize] = (a + b) as u8;
    }
    fn sub_registers(&mut self, decoded: Decoded) {
        println!("SUB V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.registers.v[decoded.x as usize];
        let b = self.registers.v[decoded.y as usize];
        if a > b {
            self.registers.v[0xF] = 1;
            self.registers.v[decoded.x as usize] = a - b;
        } else {
            self.registers.v[0xF] = 0;
            self.registers.v[decoded.x as usize] = ((256 + a as u16) - b as u16) as u8;
        }
    }
    fn shift_registers_right(&mut self, decoded: Decoded) {
        println!("SHR V{:x} V{:x}", decoded.x, decoded.y);
        if self.registers.v[decoded.x as usize] & 0x1 == 1 {
            self.registers.v[0xF] = 1;
        } else {
            self.registers.v[0xF] = 0;
        }
        self.registers.v[decoded.x as usize] = self.registers.v[decoded.x as usize] >> 1;
    }
    fn shift_registers_left(&mut self, decoded: Decoded) {
        println!("SHL V{:x} V{:x}", decoded.x, decoded.y);
        if self.registers.v[decoded.x as usize] & 0x80 == 0x80 {
            self.registers.v[0xF] = 1;
        } else {
            self.registers.v[0xF] = 0;
        }
        self.registers.v[decoded.x as usize] = self.registers.v[decoded.x as usize] << 1;
    }
    fn subn_registers(&mut self, decoded: Decoded) {
        println!("SUBN V{:x} V{:x}", decoded.x, decoded.y);
        let a = self.registers.v[decoded.x as usize];
        let b = self.registers.v[decoded.y as usize];
        if b > a {
            self.registers.v[0xF] = 1;
            self.registers.v[decoded.x as usize] = b - a;
        } else {
            self.registers.v[0xF] = 0;
            self.registers.v[decoded.x as usize] = ((256 + b as u16) - a as u16) as u8;
        }
    }
    fn set_memory_addr(&mut self, decoded: Decoded) {
        println!("SET I {:x}", decoded.nnn);
        self.registers.i = decoded.nnn;
    }
    fn jump_to_address_with_offset(&mut self, decoded: Decoded) {
        println!("JP V{:x} {:x}", 0, decoded.nnn);
        self.pc = decoded.nnn + (self.registers.v[0] as u16);
    }
    fn rnd_and(&mut self, decoded: Decoded) {
        println!("RND V{:x} {:x}", decoded.x, decoded.nn);
        self.registers.v[decoded.x as usize] = rand::random::<u8>() & decoded.nn;
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
            0x55 => self.ld_registers_memory(decoded),
            0x65 => self.ld_memory_registers(decoded),
            _ => panic!("Unknown misc op {:x}", decoded.nn),
        }
    }
    fn ld_delay_timer(&mut self, decoded: Decoded) {
        println!("LD V{:x} DT", decoded.x);
        self.registers.v[decoded.x as usize] = self.registers.delay_timer;
    }
    fn ld_delay_timer_register(&mut self, decoded: Decoded) {
        // TODO Implement decrementing of delay timer
        println!("LD DT V{:x}", decoded.x);
        self.registers.delay_timer = self.registers.v[decoded.x as usize];
    }
    fn ld_sound_timer_register(&mut self, decoded: Decoded) {
        println!("LD ST V{:x}", decoded.x);
        self.registers.sound_time = self.registers.v[decoded.x as usize];
    }
    fn add_i_register(&mut self, decoded: Decoded) {
        // handle overflow?
        println!("ADD I V{:x}", decoded.x);
        self.registers.i += self.registers.v[decoded.x as usize] as u16;
    }
    fn ld_bcd_register(&mut self, decoded: Decoded) {
        println!("LD BCD V{:x}", decoded.x);
        let num = self.registers.v[decoded.x as usize];
        self.memory
            .write(self.registers.i as usize, (num / 100) as u8);
        self.memory
            .write(self.registers.i as usize + 1, ((num % 100) / 10) as u8);
        self.memory
            .write(self.registers.i as usize + 2, (num % 10) as u8);
    }
    fn ld_registers_memory(&mut self, decoded: Decoded) {
        println!("LD V{:x} [I]", decoded.x);
        for i in 0..decoded.x {
            self.registers.v[i as usize] = self
                .memory
                .read(self.registers.i as usize + i as usize)
                .unwrap();
        }
    }
    fn ld_memory_registers(&mut self, decoded: Decoded) {
        println!("LD [I] V{:x}", decoded.x);
        for i in 0..decoded.x {
            self.memory.write(
                self.registers.i as usize + i as usize,
                self.registers.v[i as usize],
            );
        }
    }
    fn ld_font_char(&mut self, decoded: Decoded) {
        // println!("LD F V{:x}", decoded.x);
        // self.registers.i = self.registers.v[decoded.x as usize] as u16 * 5;
        panic!("Not implemented load font character");
    }

    fn ld_register_key(&mut self, decoded: Decoded) {
        panic!("Not implemented load register key");
    }
    fn skip_next_instruction_cond(&mut self, decoded: Decoded) {
        panic!("Not implemented skip cond");
    }
    fn disp_sprite(&mut self, decoded: Decoded) {
        panic!("Not implemented display sprite");
    }
    fn clear_screen(&self) {
        panic!("Not implemented clear screen");
    }
    fn return_from_subroutine(&self) {
        panic!("Not implemented return from subroutine");
    }
    fn call_subroutine(&mut self, decoded: Decoded) {
        panic!("Not implemented call subroutine");
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
        assert_eq!(cpu.pc, 0x0234);
    }
    #[test]
    fn test_skip_next_instruction_if_equal() {
        let mut cpu = CPU::default();
        cpu.registers.v[0xe] = 0x12;
        cpu.skip_next_instruction_if_equal(Decoded::new(0x3e12));
        assert_eq!(cpu.pc, 0x202);
    }
    #[test]
    fn test_skip_next_instruction_if_not_equal() {
        let mut cpu = CPU::default();
        cpu.registers.v[0xe] = 0x12;
        cpu.skip_next_instruction_if_not_equal(Decoded::new(0x4e13));
        assert_eq!(cpu.pc, 0x202);
    }
    #[test]
    fn test_apply_op() {
        let mut cpu = CPU::default();

        // Test 0x8xy0 (LD Vx, Vy)
        cpu.registers.v[0] = 0;
        cpu.registers.v[1] = 0x42;
        cpu.apply_op(Decoded::new(0x8010));
        assert_eq!(cpu.registers.v[0], 0x42);

        // Test 0x8xy1 (OR Vx, Vy)
        cpu.registers.v[0] = 0b1010;
        cpu.registers.v[1] = 0b0101;
        cpu.apply_op(Decoded::new(0x8011));
        assert_eq!(cpu.registers.v[0], 0b1111);

        // Test 0x8xy2 (AND Vx, Vy)
        cpu.registers.v[0] = 0b1010;
        cpu.registers.v[1] = 0b0110;
        cpu.apply_op(Decoded::new(0x8012));
        assert_eq!(cpu.registers.v[0], 0b0010);

        // Test 0x8xy3 (XOR Vx, Vy)
        cpu.registers.v[0] = 0b1010;
        cpu.registers.v[1] = 0b0110;
        cpu.apply_op(Decoded::new(0x8013));
        assert_eq!(cpu.registers.v[0], 0b1100);

        // Test 0x8xy4 (ADD Vx, Vy)
        cpu.registers.v[0] = 200;
        cpu.registers.v[1] = 100;
        cpu.apply_op(Decoded::new(0x8014));
        assert_eq!(cpu.registers.v[0], 44); // 300 % 256 = 44
        assert_eq!(cpu.registers.v[0xF], 1); // Carry

        cpu.registers.v[0] = 50;
        cpu.registers.v[1] = 50;
        cpu.apply_op(Decoded::new(0x8014));
        assert_eq!(cpu.registers.v[0], 100);
        assert_eq!(cpu.registers.v[0xF], 0); // No carry

        // Test 0x8xy5 (SUB Vx, Vy)
        cpu.registers.v[0] = 10;
        cpu.registers.v[1] = 5;
        cpu.apply_op(Decoded::new(0x8015));
        assert_eq!(cpu.registers.v[0], 5);
        assert_eq!(cpu.registers.v[0xF], 1); // No borrow

        cpu.registers.v[0] = 5;
        cpu.registers.v[1] = 10;
        cpu.apply_op(Decoded::new(0x8015));
        assert_eq!(cpu.registers.v[0], 251); // 256 - 5 = 251
        assert_eq!(cpu.registers.v[0xF], 0); // Borrow

        // Test 0x8xy6 (SHR Vx {, Vy})
        cpu.registers.v[0] = 0b1011;
        cpu.apply_op(Decoded::new(0x8006));
        assert_eq!(cpu.registers.v[0], 0b0101);
        assert_eq!(cpu.registers.v[0xF], 1);

        cpu.registers.v[0] = 0b1010;
        cpu.apply_op(Decoded::new(0x8006));
        assert_eq!(cpu.registers.v[0], 0b0101);
        assert_eq!(cpu.registers.v[0xF], 0);

        // Test 0x8xy7 (SUBN Vx, Vy)
        cpu.registers.v[0] = 5;
        cpu.registers.v[1] = 10;
        cpu.apply_op(Decoded::new(0x8017));
        assert_eq!(cpu.registers.v[0], 5);
        assert_eq!(cpu.registers.v[0xF], 1); // No borrow

        cpu.registers.v[0] = 10;
        cpu.registers.v[1] = 5;
        cpu.apply_op(Decoded::new(0x8017));
        assert_eq!(cpu.registers.v[0], 251); // 256 - 5 = 251
        assert_eq!(cpu.registers.v[0xF], 0); // Borrow

        // Test 0x8xyE (SHL Vx {, Vy})
        cpu.registers.v[0] = 0b01000000;
        cpu.apply_op(Decoded::new(0x800E));
        assert_eq!(cpu.registers.v[0], 0b10000000);
        assert_eq!(cpu.registers.v[0xF], 0);

        cpu.registers.v[0] = 0b10000000;
        cpu.apply_op(Decoded::new(0x800E));
        assert_eq!(cpu.registers.v[0], 0);
        assert_eq!(cpu.registers.v[0xF], 1);
    }
}
