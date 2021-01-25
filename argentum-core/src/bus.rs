//! Contains implementation of the Game Boy memory bus interface.

use crate::cartridge::*;
use crate::common::MemInterface;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::timers::Timers;

/// The Game Boy memory bus.
pub struct Bus {
    // The inserted cartridge.
    pub cartridge: Box<dyn Cartridge>,

    // 8 KB of Work RAM.
    pub wram: Box<[u8; 0x2000]>,

    // High RAM.
    pub high_ram: Box<[u8; 0x7F]>,

    // Interface to timers. (DIV, TIMA & co).
    pub timers: Timers,

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
            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.read_byte(addr),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.read_byte(addr),

            // External RAM
            // TODO
            0xA000..=0xBFFF => self.cartridge.read_byte(addr),

            // Work RAM.
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],

            // Echo RAM.
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.read_byte(addr),

            // Not Usable
            0xFEA0..=0xFEFF => 0xFF,

            // P1 or Joypad Register.
            0xFF00 => self.joypad.read_byte(addr),

            // DIV, TIMA, TMA, TAC.
            0xFF04..=0xFF07 => self.timers.read_byte(addr),

            // IF register.
            0xFF0F => self.if_flag,

            // PPU registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read_byte(addr),

            // DMA transfer request.
            0xFF46 => 0xFF,

            // High RAM.
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize],

            // IE register.
            0xFFFF => self.ie_flag,

            // Unused.
            _ => 0xFF,
        }
    }

    /// Write a byte to the specified address.
    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.write_byte(addr, value),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.write_byte(addr, value),

            // External RAM
            // TODO
            0xA000..=0xBFFF => self.cartridge.write_byte(addr, value),

            // Work RAM.
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,

            // Echo RAM.
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value,

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.write_byte(addr, value),

            // Not Usable
            0xFEA0..=0xFEFF => {}

            // P1 or Joypad Register.
            0xFF00 => self.joypad.write_byte(addr, value),

            // DIV, TIMA, TMA, TAC.
            0xFF04..=0xFF07 => self.timers.write_byte(addr, value),

            // IF register.
            0xFF0F => self.if_flag = value,

            // PPU registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.write_byte(addr, value),

            // DMA transfer request.
            0xFF46 => {
                let source = (value as u16) * 0x100;

                for i in 0..0xA0 {
                    self.write_byte(0xFE00 + i, self.read_byte(source + i));
                }
            }

            // High RAM.
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize] = value,

            // IE register.
            0xFFFF => self.ie_flag = value,

            // Unused.
            _ => {}
        }
    }
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        let cartridge: Box<dyn Cartridge> = match rom_buffer[0x0147] {
            0x00 => Box::new(RomOnly::new(rom_buffer)),
            0x0F..=0x13 => Box::new(Mbc3::new(rom_buffer)),

            _ => panic!("ROM ONLY + MBC3 cartridges are all that is currently supported."),
        };

        Self {
            cartridge,
            wram: Box::new([0; 0x2000]),
            high_ram: Box::new([0; 0x7F]),
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
