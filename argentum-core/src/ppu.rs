//! Contains implementation of the Game Boy PPU.

use crate::common::MemInterface;

/// Pallete for the framebuffer.
/// 0 - White
/// 1 - Light Gray
/// 2 - Dark Gray
/// 3 - Black
/// Alpha is FF in all cases.
const COLOR_PALLETE: [u32; 4] = [0xFFFFFFFF, 0xD3D3D3FF, 0xA9A9A9FF, 0x000000FF];

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
        // on the screen, if the window is being drawn etc..
        match self.current_mode {
            PpuModes::OamSearch if self.total_cycles >= 80 => {
                self.total_cycles -= 80;
                self.change_mode(PpuModes::Drawing);
            }

            PpuModes::Drawing if self.total_cycles >= 172 => {
                self.total_cycles -= 172;

                // Render the background map.
                self.render_background();

                self.change_mode(PpuModes::HBlank);
            }

            PpuModes::HBlank if self.total_cycles >= 204 => {
                self.total_cycles -= 204;
                self.ly += 1;

                // LY 0x90 (144) signals end of one complete frame.
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
                // These 10 lines are `psuedo lines` of sorts.
                if self.ly == 154 {
                    self.ly = 0;
                    self.change_mode(PpuModes::OamSearch);
                }
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
            0x0800
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
            // 2. Signed Mode (0x8800): (TILE_NUMBER * 16) + 0x8800.
            //
            // In the first method the TILE_NUMBER is treated as u8,
            // in the second method it is treated as i8.
            let address = if tile_map == 0x8000 {
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
            let colour = COLOR_PALLETE[pallete_index as usize];

            // Draw the pixel.
            self.draw_pixel(col, self.ly, colour);
        }
    }
}
