use crate::{audio::Apu, cartridge::*, joypad::Joypad, ppu::Ppu, timer::Timer};

mod bootrom;
mod dma;
mod interrupts;
mod speed_switch;

use self::bootrom::{CGB_BOOT_ROM, DMG_BOOT_ROM};
use self::dma::CgbDma;

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
    pub timer: Timer,

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
    pub is_cgb: bool,

    /// SVBK - WRAM Bank.
    pub wram_bank: usize,

    pub cgb_dma: CgbDma,

    /// $FF4D - KEY1.
    pub key1: u8,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom: &[u8], callback: Box<dyn Fn(&[f32])>, save_file: Option<Vec<u8>>) -> Self {
        let cartridge: Box<dyn Cartridge> = match rom[0x0147] {
            0x00 => Box::new(RomOnly::new(rom)),
            0x01..=0x03 => Box::new(Mbc1::new(rom)),
            0x0F..=0x13 => Box::new(Mbc3::new(rom, save_file)),
            0x19..=0x1E => Box::new(Mbc5::new(rom)),

            _ => panic!("unsupported cartridge type"),
        };

        let is_cgb = cartridge.has_cgb_support();

        Self {
            cartridge,
            work_ram: Box::new([0; 0x8000]),
            high_ram: Box::new([0; 0x7F]),
            timer: Timer::new(is_cgb),
            ppu: Ppu::new(is_cgb),
            apu: Apu::new(callback),
            joypad: Joypad::new(),
            ie_reg: 0,
            if_reg: 0,
            boot_reg: 0,
            is_cgb,
            wram_bank: 1,
            cgb_dma: CgbDma::new(),
            key1: 0,
        }
    }

    /// Read a byte from the given address.
    /// Tick the components if specified.
    pub fn read_byte(&mut self, addr: u16, tick: bool) -> u8 {
        let value = match addr {
            0x0000..=0x00FF if self.boot_reg == 0 => {
                if self.is_cgb {
                    CGB_BOOT_ROM[addr as usize]
                } else {
                    DMG_BOOT_ROM[addr as usize]
                }
            }

            0x0200..=0x08FF if self.boot_reg == 0 && self.is_cgb => CGB_BOOT_ROM[addr as usize],

            // ROM Banks.
            0x0000..=0x7FFF => self.cartridge.read_byte(addr),

            // Video RAM, rerouted to PPU.
            0x8000..=0x9FFF => self.ppu.read_byte(addr),

            // External RAM
            0xA000..=0xBFFF => self.cartridge.read_byte(addr),

            // Work RAM and Echo RAM
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.work_ram[(addr & 0xFFF) as usize],

            // Work RAM Bank 1~7 and Echo RAM
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.work_ram[(addr & 0xFFF) as usize + (0x1000 * self.wram_bank)]
            }

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.read_byte(addr),

            // Not Usable
            0xFEA0..=0xFEFF => 0xFF,

            // P1 - JOYP register.
            0xFF00 => self.joypad.read_byte(addr),

            // DIV, TIMA and co.
            0xFF04..=0xFF07 => self.timer.read_byte(addr),

            // IF register.
            0xFF0F => self.if_reg,

            // APU's IO registers.
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.read_byte(addr),

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F | 0xFF68 | 0xFF69..=0xFF6B => {
                self.ppu.read_byte(addr)
            }

            // DMA transfer request.
            0xFF46 => 0xFF,

            0xFF4D => self.key1,

            0xFF50 => {
                if self.boot_reg != 0 {
                    0xFF
                } else {
                    0x00
                }
            }

            // HDMA5.
            0xFF51..=0xFF55 if self.is_cgb => self.cgb_dma.read_byte(addr),

            // SVBK.
            0xFF70 if self.is_cgb => self.wram_bank as u8,

            // High RAM.
            0xFF80..=0xFFFE => self.high_ram[(addr - 0xFF80) as usize],

            // IE register.
            0xFFFF => self.ie_reg,

            _ => 0xFF,
        };

        if tick {
            self.tick_components(4);
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

            // Work RAM and Echo RAM
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.work_ram[(addr & 0xFFF) as usize] = value,

            // Work RAM Bank 1~7 and Echo RAM
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.work_ram[(addr & 0xFFF) as usize + (0x1000 * self.wram_bank)] = value;
            }

            // OAM RAM, rerouted to PPU.
            0xFE00..=0xFE9F => self.ppu.write_byte(addr, value),

            // Not Usable
            0xFEA0..=0xFEFF => {}

            // P1 - JOYP register.
            0xFF00 => self.joypad.write_byte(addr, value),

            // DIV, TIMA and co.
            0xFF04..=0xFF07 => self.timer.write_byte(addr, value),

            // IF register.
            0xFF0F => self.if_reg = value,

            // APU's IO registers.
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.write_byte(addr, value),

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F | 0xFF68 | 0xFF69..=0xFF6B => {
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

            0xFF4D => self.key1 = value & 0b0000_0001,

            // BOOT register.
            0xFF50 => {
                if self.boot_reg == 0 {
                    self.boot_reg = value;
                }
            }

            0xFF51..=0xFF55 if self.is_cgb => self.cgb_dma.write_byte(addr, value),

            0xFF70 if self.is_cgb => {
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
            self.tick_components(4);
        }
    }

    /// Skip the bootrom, and initialize all the registers.
    pub fn skip_bootrom(&mut self) {
        self.write_byte(0xFF40, 0x91, false);
        self.write_byte(0xFF47, 0xFC, false);
        self.write_byte(0xFF48, 0xFF, false);
        self.write_byte(0xFF49, 0xFF, false);

        self.boot_reg = 1;

        self.timer.skip_bootrom();
    }

    /// Tick the components on the Bus.
    pub fn tick_components(&mut self, cycles: u32) {
        let relative_cycles = cycles >> (self.is_double_speed() as u8);

        self.timer.tick(&mut self.if_reg, cycles);
        self.apu.tick(relative_cycles);
        self.joypad.update_interrupt_state(&mut self.if_reg);

        let hblank = self.ppu.tick(&mut self.if_reg, relative_cycles);
        self.tick_cgb_dma(hblank);
    }
}
