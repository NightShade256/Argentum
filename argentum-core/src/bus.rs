use alloc::boxed::Box;

use crate::{audio::Apu, cartridge::*, joypad::Joypad, ppu::Ppu, timers::Timers};

/// This is the custom copyright free bootrom for DMG
/// made by Optix.
const BOOT_ROM: &[u8] = include_bytes!("bootrom/bootix_dmg.bin");

/// Implementation of the Game Boy memory bus.
pub struct Bus {
    // The inserted cartridge.
    pub cartridge: Box<dyn Cartridge>,

    // 8 KB of Work RAM.
    pub work_ram: Box<[u8; 0x8000]>,

    // High RAM.
    pub high_ram: Box<[u8; 0x7F]>,

    /// The Game Boy timer apparatus.
    /// DIV, TIMA and co.
    pub timers: Timers,

    /// The Game Boy PPU.
    /// Contains VRAM, OAM RAM and drawing logic.
    pub ppu: Ppu,

    /// The Game Boy APU.
    /// Contains NR** registers.
    pub apu: Apu,

    /// The Game Boy joypad subsystem.
    pub joypad: Joypad,

    /// $FF0F - IF register. (Set bits here to request interrupts).
    pub if_reg: u8,

    /// $FFFF - IE register. (Set bits here to enable interrupts).
    pub ie_reg: u8,

    /// $FF50 - BOOT register. Set to non-zero value to un-map bootrom.
    pub boot_reg: u8,

    /// Is CGB mode enabled or not.
    pub cgb_mode: bool,

    /// SVBK - WRAM Bank.
    pub wram_bank: usize,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom: &[u8], callback: Box<dyn Fn(&[f32])>) -> Self {
        log::info!("ROM Information:");

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

        let cgb_mode = cartridge.has_cgb_support();

        log::info!("ROM Title: {}", cartridge.game_title());
        log::info!("CGB Mode: {}", cgb_mode);

        Self {
            cartridge,
            work_ram: Box::new([0; 0x8000]),
            high_ram: Box::new([0; 0x7F]),
            timers: Timers::new(),
            ppu: Ppu::new(cgb_mode),
            apu: Apu::new(callback),
            joypad: Joypad::new(),
            ie_reg: 0,
            if_reg: 0,
            boot_reg: 0,
            cgb_mode,
            wram_bank: 1,
        }
    }

    /// Read a byte from the given address.
    /// Tick the components if specified.
    pub fn read_byte(&mut self, addr: u16, tick: bool) -> u8 {
        let value = match addr {
            // First 256 bytes map to bootrom.
            0x0000..=0x00FF if self.boot_reg == 0 => BOOT_ROM[addr as usize],

            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.read_byte(addr),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.read_byte(addr),

            // External RAM
            0xA000..=0xBFFF => self.cartridge.read_byte(addr),

            // Work RAM.
            0xC000..=0xCFFF => self.work_ram[(addr - 0xC000) as usize],

            // Work RAM Bank 1~7
            0xD000..=0xDFFF => {
                if self.cgb_mode {
                    self.work_ram[(addr - 0xD000) as usize + (0x1000 * self.wram_bank)]
                } else {
                    self.work_ram[(addr - 0xC000) as usize]
                }
            }

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

            // APU's IO registers.
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.read_byte(addr),

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F | 0xFF68 | 0xFF69..=0xFF6C => {
                self.ppu.read_byte(addr)
            }

            // DMA transfer request.
            0xFF46 => 0xFF,

            0xFF50 => {
                if self.boot_reg != 0 {
                    0xFF
                } else {
                    0x00
                }
            }

            0xFF70 if self.cgb_mode => self.wram_bank as u8,

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
            // First 256 bytes map to bootrom.
            0x0000..=0x00FF if self.boot_reg == 0 => {}

            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.write_byte(addr, value),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.write_byte(addr, value),

            // External RAM
            0xA000..=0xBFFF => self.cartridge.write_byte(addr, value),

            // Work RAM.
            0xC000..=0xCFFF => self.work_ram[(addr - 0xC000) as usize] = value,

            // Work RAM Bank 1~7
            0xD000..=0xDFFF => {
                if self.cgb_mode {
                    self.work_ram[(addr - 0xD000) as usize + (0x1000 * self.wram_bank)] = value;
                } else {
                    self.work_ram[(addr - 0xC000) as usize] = value;
                }
            }

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

            // APU's IO registers.
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.write_byte(addr, value),

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F | 0xFF68 | 0xFF69..=0xFF6C => {
                self.ppu.write_byte(addr, value);
            }

            // DMA transfer request.
            0xFF46 => {
                let source = (value as u16) * 0x100;

                for i in 0..0xA0 {
                    let byte = self.read_byte(source + i, false);

                    self.write_byte(0xFE00 + i, byte, false);
                }
            }

            0xFF50 => {
                if self.boot_reg == 0 {
                    self.boot_reg = value;
                }
            }

            0xFF70 if self.cgb_mode => {
                let bank = (value & 0b111) as usize;

                self.wram_bank = if bank == 0 { 1 } else { bank }
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

        self.boot_reg = 1;
    }

    /// Tick the components on the Bus.
    pub fn tick(&mut self) {
        self.timers.tick(&mut self.if_reg);
        self.joypad.tick(&mut self.if_reg);
        self.ppu.tick(&mut self.if_reg);
        self.apu.tick();
    }
}
