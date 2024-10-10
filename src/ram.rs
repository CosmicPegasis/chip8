use std::fs::File;
use std::io::{self, Read};

struct RAM {
    cart: [u8; 4096],
    cart_size: usize,
}

impl RAM {
    fn default() -> Self {
        return RAM {
            cart: [0; 4096],
            cart_size: 0,
        };
    }
    fn load(&mut self, path: &str) -> io::Result<()> {
        let mut file = File::open(path)?;

        // buffer is a maximum of 4096 which is the same as my cartridge
        let mut buffer = vec![0; 4096];
        let bytes_read = file.read(&mut buffer)?;

        self.cart[0x200..(0x200 + bytes_read)].copy_from_slice(&buffer[..bytes_read]);

        self.cart_size = bytes_read;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glitch_ghost_load_test() {
        // Will need the roms folder to run
        let mut ram = RAM::default();

        ram.load("roms/glitchGhost.ch8").unwrap();
        assert_eq!(ram.cart_size, 2907);
        assert_eq!(ram.cart[0x200], 0x1c);
    }
}
