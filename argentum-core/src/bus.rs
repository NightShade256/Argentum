use alloc::boxed::Box;

pub struct Bus {
    memory: Box<[u8; 0x10000]>,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom: &[u8]) -> Self {
        let mut bus = Self {
            memory: Box::new([0; 0x10000]),
        };

        bus.memory[0x0000..=0x7FFF].copy_from_slice(rom);

        bus
    }

    /// Read a byte from the given address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        if addr == 0xFF44 {
            return 0x90;
        }

        self.memory[addr as usize]
    }

    /// Write a byte to the given address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        if addr == 0xFF44 {
            return;
        }

        self.memory[addr as usize] = value;
    }
}
