//! Contains implementation of the Game Boy memory bus interface.

use crate::common::MemInterface;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::timers::Timers;

/// The Game Boy memory bus.
pub struct Bus {
    // Ad hoc implementation.
    memory: Box<[u8; u16::MAX as usize + 1]>,

    // Interface to timers. (DIV, TIMA & co).
    timers: Timers,

    /// The joypad interface.
    pub joypad: Joypad,

    // The PPU itself.
    pub ppu: Ppu,

    // IF flag, mapped to 0xFF0F.
    pub if_flag: u8,

    // IE flag, mapped to 0xFFFF.
    pub ie_flag: u8,
}

impl MemInterface for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.read_byte(addr),

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.read_byte(addr),

            // Stub Joypad
            // TEMPORARY
            0xFF00 => self.joypad.read_byte(addr),

            // Timer IO.
            0xFF04..=0xFF07 => self.timers.read_byte(addr),

            // PPU IO.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read_byte(addr),

            // Interrupts.
            0xFF0F => self.if_flag,
            0xFFFF => self.ie_flag,

            _ => self.memory[addr as usize],
        }
    }

    /// Write a byte to the specified address.
    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => {}

            // Video RAM.
            0x8000..=0x9FFF => self.ppu.write_byte(addr, value),

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.write_byte(addr, value),

            // Joypad IO.
            0xFF00 => self.joypad.write_byte(addr, value),

            // Timer IO.
            0xFF04..=0xFF07 => self.timers.write_byte(addr, value),

            // PPU IO.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.write_byte(addr, value),

            // DMA
            0xFF46 => {
                let source = (value as u16) * 0x100;

                for i in 0..0xA0 {
                    self.write_byte(0xFE00 + i, self.read_byte(source + i));
                }
            }

            // Interrupts.
            0xFF0F => self.if_flag = value,
            0xFFFF => self.ie_flag = value,

            _ => self.memory[addr as usize] = value,
        }
    }
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
            joypad: Joypad::new(),
            ppu: Ppu::new(),
            if_flag: 0,
            ie_flag: 0,
        }
    }

    /// Skip the bootrom, and initialize all the registers.
    pub fn skip_bootrom(&mut self) {
        self.ppu.write_byte(0xFF40, 0x91);
        self.ppu.write_byte(0xFF47, 0xFC);
        self.ppu.write_byte(0xFF48, 0xFF);
        self.ppu.write_byte(0xFF49, 0xFF);
    }

    /// Tick all the components on the bus by the given T-cycles.
    pub fn tick_components(&mut self, t_elapsed: u32) {
        self.timers.tick(t_elapsed, &mut self.if_flag);
        self.ppu.tick(t_elapsed, &mut self.if_flag);
        self.joypad.tick(t_elapsed, &mut self.if_flag);
    }
}
