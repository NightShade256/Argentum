use alloc::boxed::Box;

use crate::{cartridge::*, joypad::Joypad, ppu::Ppu, timers::Timers};

/// Implementation of the Game Boy memory bus.
pub struct Bus {
    // The inserted cartridge.
    pub cartridge: Box<dyn Cartridge>,

    /// The Game Boy timer apparatus.
    /// DIV, TIMA and co.
    pub timers: Timers,

    /// The Game Boy PPU.
    /// Contains VRAM, OAM RAM and drawing logic.
    pub ppu: Ppu,

    /// The Game Boy joypad subsystem.
    pub joypad: Joypad,

    /// $FF0F - IF register. (Set bits here to request interrupts).
    pub if_reg: u8,

    /// $FFFF - IE register. (Set bits here to enable interrupts).
    pub ie_reg: u8,

    pub memory: Box<[u8; 0x10000]>,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom: &[u8]) -> Self {
        let cartridge: Box<dyn Cartridge> = match rom[0x0147] {
            0x00 => Box::new(RomOnly::new(rom)),
            0x01..=0x03 => Box::new(Mbc1::new(rom)),
            0x19..=0x1E => Box::new(Mbc5::new(rom)),

            _ => panic!("ROM ONLY + MBC1 + MBC5 cartridges are all that is currently supported."),
        };

        Self {
            cartridge,
            timers: Timers::new(),
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            ie_reg: 0,
            if_reg: 0,
            memory: Box::new([0; 0x10000]),
        }
    }

    /// Read a byte from the given address.
    /// Tick the components if specified.
    pub fn read_byte(&mut self, addr: u16, tick: bool) -> u8 {
        let value = match addr {
            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.read_byte(addr),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.read_byte(addr),

            // External RAM
            0xA000..=0xBFFF => self.cartridge.read_byte(addr),

            // P1 - JOYP register.
            0xFF00 => self.joypad.read_byte(addr),

            // DIV, TIMA and co.
            0xFF04..=0xFF07 => self.timers.read_byte(addr),

            // IF register.
            0xFF0F => self.if_reg,

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read_byte(addr),

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.read_byte(addr),

            // IE register.
            0xFFFF => self.ie_reg,

            _ => self.memory[addr as usize],
        };

        if tick {
            self.tick();
        }

        value
    }

    /// Write a byte to the given address.
    /// Tick the components if specified.
    pub fn write_byte(&mut self, addr: u16, value: u8, tick: bool) {
        match addr {
            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.write_byte(addr, value),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.write_byte(addr, value),

            // External RAM
            0xA000..=0xBFFF => self.cartridge.write_byte(addr, value),

            // P1 - JOYP register.
            0xFF00 => self.joypad.write_byte(addr, value),

            // DIV, TIMA and co.
            0xFF04..=0xFF07 => self.timers.write_byte(addr, value),

            // IF register.
            0xFF0F => self.if_reg = value,

            // PPU's IO register.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.write_byte(addr, value),

            // DMA transfer request.
            0xFF46 => {
                let source = (value as u16) * 0x100;

                for i in 0..0xA0 {
                    let byte = self.read_byte(source + i, false);

                    self.write_byte(0xFE00 + i, byte, false);
                }
            }

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.write_byte(addr, value),

            // IE register.
            0xFFFF => self.ie_reg = value,

            _ => self.memory[addr as usize] = value,
        }

        if tick {
            self.tick();
        }
    }

    /// Skip the bootrom, and initialize all the registers.
    pub fn skip_bootrom(&mut self) {
        self.write_byte(0xFF40, 0x91, false);
        self.write_byte(0xFF47, 0xFC, false);
        self.write_byte(0xFF48, 0xFF, false);
        self.write_byte(0xFF49, 0xFF, false);
    }

    /// Tick the components on the Bus.
    pub fn tick(&mut self) {
        self.timers.tick(&mut self.if_reg);
        self.joypad.tick(&mut self.if_reg);
        self.ppu.tick(&mut self.if_reg);
    }
}
