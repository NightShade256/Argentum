use std::{cell::RefCell, rc::Rc};

use crate::{audio::Apu, cartridge::*, joypad::Joypad, ppu::Ppu, timer::Timer};

/// This is a custom bootrom for DMG
/// made by LIJI.
static DMG_BOOT_ROM: &[u8] = include_bytes!("bootrom/dmg_boot.bin");

/// This is a custom bootrom for CGB
/// made by LIJI.
static CGB_BOOT_ROM: &[u8] = include_bytes!("bootrom/cgb_boot.bin");

/// Implementation of the Game Boy memory bus.
pub(crate) struct Bus {
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
    pub if_reg: Rc<RefCell<u8>>,

    /// $FFFF - IE register. (Set bits here to enable interrupts).
    pub ie_reg: u8,

    /// $FF50 - BOOT register. Set to non-zero value to un-map bootrom.
    pub boot_reg: u8,

    /// Is CGB mode enabled or not.
    pub cgb_mode: bool,

    /// SVBK - WRAM Bank.
    pub wram_bank: usize,

    /// $FF51 - HDMA1
    pub dma_src_high: u8,

    /// $FF52 - HDMA2
    pub dma_src_low: u8,

    /// $FF53 - HDMA3
    pub dma_dst_high: u8,

    /// $FF54 - HDMA4
    pub dma_dst_low: u8,

    /// $FF55 - HDMA5
    pub dma_control: u8,

    /// Signals whether HDMA is currently active.
    pub hdma_active: bool,

    /// The remaining length of the HDMA transfer.
    pub hdma_len: u16,

    /// The HDMA sourcefrom where to read the next byte.
    pub hdma_src: u16,

    /// The HDMA destination where to transfer the next byte.
    pub hdma_dst: u16,

    /// $FF4D - KEY1.
    pub speed_reg: u8,
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

        let if_reg = Rc::new(RefCell::new(0));
        let cgb_mode = cartridge.has_cgb_support();

        Self {
            cartridge,
            work_ram: Box::new([0; 0x8000]),
            high_ram: Box::new([0; 0x7F]),
            timer: Timer::new(Rc::clone(&if_reg)),
            ppu: Ppu::new(Rc::clone(&if_reg), cgb_mode),
            apu: Apu::new(callback),
            joypad: Joypad::new(Rc::clone(&if_reg)),
            ie_reg: 0,
            if_reg,
            boot_reg: 0,
            cgb_mode,
            wram_bank: 1,
            dma_src_high: 0,
            dma_src_low: 0,
            dma_dst_high: 0,
            dma_dst_low: 0,
            dma_control: 0,
            hdma_active: false,
            hdma_len: 0,
            hdma_dst: 0,
            hdma_src: 0,
            speed_reg: 0,
        }
    }

    /// Read a byte from the given address.
    /// Tick the components if specified.
    pub fn read_byte(&mut self, addr: u16, tick: bool) -> u8 {
        let value = match addr {
            0x0000..=0x00FF if self.boot_reg == 0 => {
                if self.cgb_mode {
                    CGB_BOOT_ROM[addr as usize]
                } else {
                    DMG_BOOT_ROM[addr as usize]
                }
            }

            0x0200..=0x08FF if self.boot_reg == 0 && self.cgb_mode => CGB_BOOT_ROM[addr as usize],

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
            0xFF0F => *self.if_reg.borrow(),

            // APU's IO registers.
            0xFF10..=0xFF26 | 0xFF30..=0xFF3F => self.apu.read_byte(addr),

            // PPU's IO registers.
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F | 0xFF68 | 0xFF69..=0xFF6B => {
                self.ppu.read_byte(addr)
            }

            // DMA transfer request.
            0xFF46 => 0xFF,

            0xFF4D => self.speed_reg,

            0xFF50 => {
                if self.boot_reg != 0 {
                    0xFF
                } else {
                    0x00
                }
            }

            // HDMA5.
            0xFF55 if self.cgb_mode => self.dma_control,

            // SVBK.
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
            0xFF0F => *self.if_reg.borrow_mut() = value,

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

            0xFF4D => self.speed_reg = value & 0b0000_0001,

            // BOOT register.
            0xFF50 => {
                if self.boot_reg == 0 {
                    self.boot_reg = value;
                }
            }

            // HDMA1
            0xFF51 if self.cgb_mode => {
                self.dma_src_high = value;
            }

            // HDMA2
            0xFF52 if self.cgb_mode => {
                self.dma_src_low = value;
            }

            // HDMA3
            0xFF53 if self.cgb_mode => {
                self.dma_dst_high = value;
            }

            // HDMA4
            0xFF54 if self.cgb_mode => {
                self.dma_dst_low = value;
            }

            // HDMA5
            0xFF55 if self.cgb_mode => {
                // The length of the DMA.
                let len = (((value & 0x7F) as u16) + 1) << 4;

                // Source and destination addresses of the DMA.
                let src = ((self.dma_src_high as u16) << 8) | ((self.dma_src_low & 0xF0) as u16);
                let dst = ((self.dma_dst_high as u16) << 8) | ((self.dma_dst_low & 0xF0) as u16);

                // Check if the DMA is a GDMA or a HDMA.
                if (value & 0x80) != 0 {
                    self.dma_control = value;

                    self.hdma_len = len;
                    self.hdma_dst = dst;
                    self.hdma_src = src;
                    self.hdma_active = true;
                } else {
                    // If HDMA was activated earlier and top bit is 0 it means
                    // instead of GDMA the ROM wants to cancel the earlier DMA.
                    if self.hdma_active {
                        self.hdma_active = false;
                        self.dma_control = 0xFF;
                        return;
                    }

                    // We copy all the bytes instantly and not worry about
                    // timings currently.
                    for i in 0..len {
                        let value = self.read_byte(src + i, false);

                        self.write_byte(dst + i, value, false);
                    }

                    self.dma_control = 0xFF;
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

    /// Check if we are in double speed mode.
    pub fn is_double_speed(&self) -> bool {
        (self.speed_reg & 0b1000_0000) != 0
    }

    /// Tick the components on the Bus.
    pub fn tick(&mut self) {
        let cycles = 4 >> (self.is_double_speed() as u8);

        self.timer.tick();
        self.apu.tick(cycles);

        let entered_hblank = self.ppu.tick(cycles);

        // If we entered HBlank and HDMA is active perform
        // a transfer of 0x10 bytes.
        if entered_hblank && self.hdma_active {
            for i in 0..0x10 {
                let byte = self.read_byte(self.hdma_src + i, false);

                // The destination is always in VRAM.
                self.ppu
                    .write_byte(((self.hdma_dst + i) & 0x1FFF) + 0x8000, byte);
            }

            self.hdma_len -= 0x10;
            self.hdma_src += 0x10;
            self.hdma_dst += 0x10;

            self.dma_control -= 1;

            // Switch off HDMA if all bytes are transferred.
            if self.hdma_len == 0 {
                self.dma_control = 0xFF;
                self.hdma_active = false;
            }
        }
    }
}
