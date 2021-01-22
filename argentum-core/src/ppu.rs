//! Contains implementation of the Game Boy PPU.

use crate::common::MemInterface;

/// Pallete for the framebuffer.
/// 0 - White
/// 1 - Light Gray
/// 2 - Dark Gray
/// 3 - Black
/// Alpha is FF in all cases.
///
/// Palette is equal to the palette of BGB.
const COLOR_PALETTE: [u32; 4] = [0xE0F8D0FF, 0x88C070FF, 0x346856FF, 0x081820FF];

/// Enumerates all the different modes the PPU can be in.
pub enum PpuModes {
    HBlank,
    VBlank,
    OamSearch,
    Drawing,
}

/// Implementation of the Game Boy PPU.
pub struct Ppu {
    /// 8 KB VRAM
    /// Mapped to 0x8000 to 0x9FFF.
    vram: Box<[u8; 0x2000]>,

    /// The current scanline the PPU is rendering.
    ly: u8,

    /// LY Compare register.
    lyc: u8,

    /// The LCD control register.
    lcdc: u8,

    /// LCD status register.
    stat: u8,

    /// Scroll Y register.
    scy: u8,

    /// Scroll X register.
    scx: u8,

    /// Background pallete data.
    bgp: u8,

    /// The current mode the PPU is in.
    current_mode: PpuModes,

    /// Total cycles ticked under the current mode.
    total_cycles: u32,

    /// RGBA32 framebuffer, to be rendered by the frontend.
    pub framebuffer: Box<[u8; 160 * 144 * 4]>,
}

impl MemInterface for Ppu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = (value & 0xFC) | (self.stat & 0x07),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {}
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,

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
            lyc: 0,
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            bgp: 0,
            current_mode: PpuModes::OamSearch,
            total_cycles: 0,
            framebuffer: Box::new([0; 160 * 144 * 4]),
        }
    }

    /// Change the PPU's current mode.
    fn change_mode(&mut self, mode: PpuModes, if_reg: &mut u8) {
        match &mode {
            PpuModes::HBlank => {
                self.current_mode = mode;
                self.stat &= 0xFC;

                // Render the scanline.
                // Right now it's just the background.
                self.render_background();

                // Request STAT interrupt if
                // the appropriate bit is set.
                if (self.stat & 0x08) != 0 {
                    *if_reg |= 0b0000_0010;
                }
            }

            PpuModes::VBlank => {
                self.current_mode = mode;
                self.stat = (self.stat & 0xFC) | 0x01;

                // Request VBlank interrupt.
                *if_reg |= 0b0000_0001;

                // Request STAT interrupt if
                // the appropriate bit is set.
                if (self.stat & 0x10) != 0 {
                    *if_reg |= 0b0000_0010;
                }
            }

            PpuModes::OamSearch => {
                self.current_mode = mode;
                self.stat = (self.stat & 0xFC) | 0x02;

                // Request STAT interrupt if
                // the appropriate bit is set.
                if (self.stat & 0x20) != 0 {
                    *if_reg |= 0b0000_0010;
                }
            }

            PpuModes::Drawing => {
                self.current_mode = mode;
                self.stat = (self.stat & 0xFC) | 0x03;
            }
        }
    }

    /// Compare LY and LYC, set bits and trigger interrupts.
    fn compare_lyc(&mut self, if_reg: &mut u8) {
        if self.ly == self.lyc {
            self.stat |= 0x04;

            if (self.stat & 0x40) != 0 {
                *if_reg |= 0b0000_0010;
            }
        } else {
            self.stat &= !0x04;
        }
    }

    /// Tick the PPU by the given T-cycles.
    pub fn tick(&mut self, t_elapsed: u32, if_reg: &mut u8) {
        self.total_cycles += t_elapsed;

        // The actual PPU timings are not fixed.
        // They vary depending upon the number of sprites
        // on the screen, if the window is being drawn etc..
        match self.current_mode {
            PpuModes::OamSearch if self.total_cycles >= 80 => {
                self.total_cycles -= 80;
                self.change_mode(PpuModes::Drawing, if_reg);
            }

            PpuModes::Drawing if self.total_cycles >= 172 => {
                self.total_cycles -= 172;
                self.change_mode(PpuModes::HBlank, if_reg);
            }

            PpuModes::HBlank if self.total_cycles >= 204 => {
                self.total_cycles -= 204;
                self.ly += 1;

                // LY 0x90 (144) signals end of one complete frame.
                if self.ly == 0x90 {
                    self.change_mode(PpuModes::VBlank, if_reg);
                } else {
                    self.change_mode(PpuModes::OamSearch, if_reg);
                }

                self.compare_lyc(if_reg);
            }

            PpuModes::VBlank if self.total_cycles >= 456 => {
                self.total_cycles -= 456;
                self.ly += 1;

                // The PPU actually has 154 lines instead of 144.
                // These 10 lines are `psuedo lines` of sorts.
                if self.ly == 154 {
                    self.ly = 0;
                    self.change_mode(PpuModes::OamSearch, if_reg);
                }

                self.compare_lyc(if_reg);
            }

            _ => {}
        }
    }

    /// Draw a pixel in the framebuffer at the given `x` and `y`
    /// coordinates.
    fn draw_pixel(&mut self, x: u8, y: u8, colour: u32) {
        let offset = (y as usize * 160 * 4) + x as usize * 4;
        let bytes = colour.to_be_bytes();

        self.framebuffer[offset..offset + 4].copy_from_slice(&bytes);
    }

    // Render the background map with scroll.
    // Windows not yet rendered.
    fn render_background(&mut self) {
        // This kind of disables windows and background
        // drawing.
        // Sprites not affected by this.
        if (self.lcdc & 0x01) == 0 {
            return;
        }

        // The address of the tile map that is to
        // be rendered minus 0x8000.
        let tile_map: u16 = if (self.lcdc & 0x08) != 0 {
            0x1C00
        } else {
            0x1800
        };

        // The address of the tile data that
        // is going to be used for rendering minus 0x8000.
        let tile_data: u16 = if (self.lcdc & 0x10) != 0 {
            0x0000
        } else {
            0x1000
        };

        // The Y coordinate we are interested in, in the
        // background tile map.
        let y = self.ly.wrapping_add(self.scy);

        for col in 0u8..160u8 {
            // The X coordinate we are interested in, in the
            // background tile map.
            let x = col.wrapping_add(self.scx);

            // What tile number the coordinates correspond to.
            // Each tile is 8 pixels wide and 8 pixels tall, and
            // the background is 32 by 32 tiles in size.
            let tile_number_index = tile_map + (((y as u16 / 8) * 32) + (x as u16 / 8));
            let tile_number = self.vram[tile_number_index as usize];

            // The row of the tile we are interested in.
            // (Each row is of two bytes).
            let row = ((y % 8) * 2) as u16;

            // Here we get the tile address.
            // There are two addressing modes.
            // 1. Unsigned Mode (0x8000): (TILE_NUMBER * 16) + 0x8000.
            // 2. Signed Mode (0x8800): (TILE_NUMBER * 16) + 09000.
            //
            // In the first method the TILE_NUMBER is treated as u8,
            // in the second method it is treated as i8.
            let address = if tile_data == 0x0000 {
                // Unsigned addressing mode.
                tile_data + (tile_number as u16 * 16) + row
            } else {
                // Signed addressing mode.
                tile_data + ((tile_number as i8 as i16) as u16 * 16) + row
            } as usize;

            // The two bytes of the row.
            // Remember LITTLE ENDIAN.
            let byte_two = self.vram[address];
            let byte_one = self.vram[address + 1];

            // The colour pallete index.
            let col_ = x % 8;
            let colour =
                (((byte_two >> (7 - col_)) & 0x01) << 1) | ((byte_one >> (7 - col_)) & 0x01);
            let pallete_index = (self.bgp >> (colour << 1)) & 0x03;

            // The actual colour of the pixel.
            let colour = COLOR_PALETTE[pallete_index as usize];

            // Draw the pixel.
            self.draw_pixel(col, self.ly, colour);
        }
    }
}
