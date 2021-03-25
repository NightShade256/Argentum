use alloc::boxed::Box;

use crate::{audio::Apu, cartridge::*, joypad::Joypad, ppu::Ppu, timers::Timers};

/// Implementation of the Game Boy memory bus.
pub struct Bus {
    // The inserted cartridge.
    pub cartridge: Box<dyn Cartridge>,

    // 8 KB of Work RAM.
    pub work_ram: Box<[u8; 0x2000]>,

    // High RAM.
    pub high_ram: Box<[u8; 0x7F]>,

    /// The Game Boy timer apparatus.
    /// DIV, TIMA and co.
    pub timers: Timers,

    /// The Game Boy PPU.
    /// Contains VRAM, OAM RAM and drawing logic.
    pub ppu: Ppu,

    pub apu: Apu,

    /// The Game Boy joypad subsystem.
    pub joypad: Joypad,

    /// $FF0F - IF register. (Set bits here to request interrupts).
    pub if_reg: u8,

    /// $FFFF - IE register. (Set bits here to enable interrupts).
    pub ie_reg: u8,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom: &[u8]) -> Self {
        log::info!("ROM Information...");

        let cartridge: Box<dyn Cartridge> = match rom[0x0147] {
            0x00 => {
                log::info!("Cartridge Type: ROM Only.");
                Box::new(RomOnly::new(rom))
            }
            0x01..=0x03 => {
                log::info!("Cartridge Type: MBC1.");
                Box::new(Mbc1::new(rom))
            }
            0x0F..=0x13 => {
                log::info!("Cartridge Type: MBC3.");
                Box::new(Mbc3::new(rom))
            }
            0x19..=0x1E => {
                log::info!("Cartridge Type: MBC5.");
                Box::new(Mbc5::new(rom))
            }
            _ => panic!("ROM ONLY + MBC(1/3/5) cartridges are all that is currently supported."),
        };

        log::info!("Title: {}", cartridge.game_title());

        Self {
            cartridge,
            work_ram: Box::new([0; 0x2000]),
            high_ram: Box::new([0; 0x7F]),
            timers: Timers::new(),
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            ie_reg: 0,
            if_reg: 0,

            apu: Apu::new(),
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

            // Work RAM.
            0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize],

            // Echo RAM.
            0xE000..=0xFDFF => self.work_ram[(addr - 0xE000) as usize],

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.read_byte(addr),

            // Not Usable
            0xFEA0..=0xFEFF => 0xFF,

            // P1 - JOYP register.
            0xFF00 => self.joypad.read_byte(addr),

            // DIV, TIMA and co.
            0xFF04..=0xFF07 => self.timers.read_byte(addr),

            // IF register.
            0xFF0F => self.if_reg,

            // NR50 register.
            0xFF24..=0xFF26 | 0xFF16..=0xFF19 => self.apu.read_byte(addr),

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read_byte(addr),

            // DMA transfer request.
            0xFF46 => 0xFF,

            // High RAM.
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize],

            // IE register.
            0xFFFF => self.ie_reg,

            _ => 0xFF,
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

            // Work RAM.
            0xC000..=0xDFFF => self.work_ram[(addr - 0xC000) as usize] = value,

            // Echo RAM.
            0xE000..=0xFDFF => self.work_ram[(addr - 0xE000) as usize] = value,

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.write_byte(addr, value),

            // Not Usable
            0xFEA0..=0xFEFF => {}

            // P1 - JOYP register.
            0xFF00 => self.joypad.write_byte(addr, value),

            // DIV, TIMA and co.
            0xFF04..=0xFF07 => self.timers.write_byte(addr, value),

            // IF register.
            0xFF0F => self.if_reg = value,

            // NR50 register.
            0xFF24..=0xFF26 | 0xFF16..=0xFF19 => self.apu.write_byte(addr, value),

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

            // High RAM.
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize] = value,

            // IE register.
            0xFFFF => self.ie_reg = value,

            _ => {}
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
        self.apu.tick();
    }
}
