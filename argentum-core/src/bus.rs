//! Contains implementation of the Game Boy memory bus interface.

use crate::timers::Timers;

/// The Game Boy memory bus.
pub struct Bus {
    // Ad hoc implementation.
    memory: Box<[u8; u16::MAX as usize + 1]>,

    // Interface to timers. (DIV, TIMA & co).
    timers: Timers,

    // IF flag, mapped to 0xFF0F.
    pub if_flag: u8,

    // IE flag, mapped to 0xFFFF.
    pub ie_flag: u8,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        let mut memory = Box::new([0; u16::MAX as usize + 1]);

        // Load the ROM into memory.
        memory[..rom_buffer.len()].copy_from_slice(rom_buffer);

        Self {
            memory,
            timers: Timers::new(),
            if_flag: 0,
            ie_flag: 0,
        }
    }

    /// Tick all the components on the bus by the given T-cycles.
    pub fn tick_components(&mut self, t_elapsed: u32) {
        self.timers.tick(t_elapsed, &mut self.if_flag);
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Timer IO.
            0xFF04..=0xFF07 => self.timers.read(addr),

            0xFF0F => self.if_flag,

            // Stub LY to 0x90 temporary.
            0xFF44 => 0x90,

            0xFFFF => self.ie_flag,

            _ => self.memory[addr as usize],
        }
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // Timer IO.
            0xFF04..=0xFF07 => self.timers.write(addr, value),

            0xFF0F => self.if_flag = value,

            // Stub LY to 0x90 temporary.
            0xFF44 => {}

            0xFFFF => self.ie_flag = value,

            _ => self.memory[addr as usize] = value,
        }
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
