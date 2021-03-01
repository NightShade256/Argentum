use alloc::boxed::Box;

pub struct Bus {
    memory: Box<[u8; 0x10000]>,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new() -> Self {
        Self {
            memory: Box::new([0; 0x10000]),
        }
    }

    /// Read a byte from the given address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    /// Write a byte to the given address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}
