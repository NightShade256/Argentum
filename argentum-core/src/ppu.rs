//! Contains implementation of the Game Boy PPU.

use crate::common::MemInterface;

pub enum PpuModes {
    HBlank,
    VBlank,
    OamSearch,
    Drawing,
}

pub struct Ppu {
    /// 8 KB VRAM
    /// Mapped to 0x8000 to 0x9FFF.
    vram: Box<[u8; 0x2000]>,

    /// The current scanline the PPU is rendering.
    ly: u8,

    /// The LCD control register.
    lcdc: u8,

    /// LCD status register.
    stat: u8,

    /// The current mode the PPU is in.
    current_mode: PpuModes,

    /// Total cycles ticked under the current mode.
    total_cycles: u32,

    /// RGBA32 framebuffer, to be rendered by the frontend.
    framebuffer: Box<[u8; 160 * 144 * 4]>,
}

impl MemInterface for Ppu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF44 => self.ly,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = (value & 0xFC) | (self.stat & 0x07),
            0xFF44 => {}

            _ => unreachable!(),
        }
    }
}

impl Ppu {
    /// Create a new `Ppu` instance.
    pub fn new() -> Self {
        Self {
            vram: Box::new([0; 0x2000]),
            ly: 0,
            lcdc: 0,
            stat: 0,
            current_mode: PpuModes::OamSearch,
            total_cycles: 0,
            framebuffer: Box::new([0; 160 * 144 * 4]),
        }
    }

    /// Change the PPU's mode.
    pub fn change_mode(&mut self, mode: PpuModes) {
        match &mode {
            PpuModes::HBlank => {
                self.current_mode = mode;
                self.stat &= 0xFC;
            }

            PpuModes::VBlank => {
                self.current_mode = mode;
                self.stat = (self.stat & 0xFC) | 0x01;
            }

            PpuModes::OamSearch => {
                self.current_mode = mode;
                self.stat = (self.stat & 0xFC) | 0x02;
            }

            PpuModes::Drawing => {
                self.current_mode = mode;
                self.stat = (self.stat & 0xFC) | 0x03;
            }
        }
    }

    /// Tick the PPU by the given T-cycles.
    pub fn tick(&mut self, t_elapsed: u32) {
        self.total_cycles += t_elapsed;

        // The actual PPU timings are not fixed.
        // They vary depending upon the number of sprites
        // on screen, window etc..
        match self.current_mode {
            PpuModes::OamSearch if self.total_cycles >= 80 => {
                self.total_cycles -= 80;
                self.change_mode(PpuModes::Drawing);
            }

            PpuModes::Drawing if self.total_cycles >= 172 => {
                self.total_cycles -= 172;
                self.change_mode(PpuModes::HBlank);
            }

            PpuModes::HBlank if self.total_cycles >= 204 => {
                self.total_cycles -= 204;
                self.ly += 1;

                if self.ly == 0x90 {
                    self.change_mode(PpuModes::VBlank);
                } else {
                    self.change_mode(PpuModes::OamSearch);
                }
            }

            PpuModes::VBlank if self.total_cycles >= 456 => {
                self.total_cycles -= 456;
                self.ly += 1;

                // The PPU actually has 154 lines instead of 144.
                if self.ly == 154 {
                    self.ly = 0;
                    self.change_mode(PpuModes::OamSearch);
                }
            }

            _ => {}
        }
    }

    // For now it just renders the BG map as it is.
    pub fn render_background(&mut self) {}
}
