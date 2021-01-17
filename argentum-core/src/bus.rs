//! Contains implementation of the Game Boy memory bus interface.

/// The Game Boy memory bus.
pub struct Bus {
    // Ad hoc implementation.
    memory: Box<[u8; u16::MAX as usize + 1]>,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        let mut memory = Box::new([0; u16::MAX as usize + 1]);

        // Load the ROM into memory.
        memory[..rom_buffer.len()].copy_from_slice(rom_buffer);

        Self { memory }
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        // Stub LY to 0x90
        // Temporary.
        if addr == 0xFF44 {
            return 0x90;
        }

        self.memory[addr as usize]
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        if addr == 0xFF44 {
            return;
        }

        self.memory[addr as usize] = value;
    }

    /// Read a word from the specified address.
    pub fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    /// Write a word to the specified address.
    pub fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }
}
