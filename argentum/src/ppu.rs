use std::{cell::RefCell, rc::Rc};

use crate::util::{bit, res, set};

/// The colour palette used in DMG mode.
/// 0 - White
/// 1 - Light Gray
/// 2 - Dark Gray
/// 3 - Black
const DMG_MODE_PALETTE: [u32; 4] = [0xFED018, 0xD35600, 0x5E1210, 0x0D0405];

/// Represents sprite data as stored in OAM.
#[derive(Clone, Copy)]
#[repr(C)]
struct Sprite {
    /// The absolute Y coordinate of the sprite.
    y: u8,

    /// The absolute X coordinate of the sprite.
    x: u8,

    /// The tile index of the sprite.
    index: u8,

    /// The sprite attributes and flags.
    flags: u8,
}

impl From<[u8; 4]> for Sprite {
    fn from(oam_entry: [u8; 4]) -> Self {
        Self {
            y: oam_entry[0],
            x: oam_entry[1],
            index: oam_entry[2],
            flags: oam_entry[3],
        }
    }
}

/// Enumerates all the different modes the PPU can be in.
#[derive(Clone, Copy)]
#[repr(u8)]
pub(crate) enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    Drawing = 3,
}

pub(crate) struct Ppu {
    /// 8 KiB of Video RAM
    ///
    /// Mapped to 0x8000 to 0x9FFF.
    /// Capacity for two complete banks, but only 1 is used
    /// in DMG mode.
    vram: [u8; 0x4000],

    /// 160 B of OAM RAM.
    ///
    /// Mapped to 0xFE00 to 0xFE9F.
    /// Space for 40 sprite entries.
    oam_ram: [u8; 0xA0],

    /// 0xFF40 - LCD Control.
    ///
    /// Controls pretty much all aspects of rendering.
    lcdc: u8,

    /// 0xFF41 - LCD Status.
    ///
    /// Contains control bits for the STAT interrupt, and
    /// indicates the current PPU mode to the game ROM.
    stat: u8,

    /// 0xFF42 - Scroll Y.
    scy: u8,

    /// 0xFF43 - Scroll X.
    scx: u8,

    /// 0xFF44 - LY.
    ///
    /// The current scanline, that is being rendered.
    /// Read only indicator to the game ROM.
    ly: u8,

    /// 0xFF45 - LY Compare.
    ///
    /// LYC is permanently compared with LY.
    /// STAT interrupt is triggered if LYC = LY.
    lyc: u8,

    /// 0xFF47 - Background Palette Data (DMG Mode Only).
    bgp: u8,

    /// 0xFF48 - Sprite Palette 0 (DMG Mode Only).
    obp0: u8,

    /// 0xFF49 - Sprite Palette 1 (DMG Mode Only).
    obp1: u8,

    /// 0xFF4A - Window Y coordinate.
    wy: u8,

    /// 0xFF4B - Window X coordinate - 7.
    wx: u8,

    /// Internal GB window line counter.
    window_line_counter: u8,

    /// Indicates whether we should emulate DMG or
    /// CGB behaviour.
    cgb_mode: bool,

    /// 0xFF68 - BCPS.
    ///
    /// Specifies the index of the byte curently
    /// selected in BCPD register.
    bcps: u8,

    /// 0xFF69 - BCPD
    ///
    /// Background Colour Palettes (CGB Mode Only).
    bgd_palettes: [u8; 0x40],

    /// 0xFF6A - BCPS.
    ///
    /// Specifies the index of the byte curently
    /// selected in BCPD register.
    ocps: u8,

    /// 0xFF69 - OCPD
    ///
    /// Sprite Colour Palettes (CGB Mode Only).
    obj_palettes: [u8; 0x40],

    /// State of background rendering for the
    /// current LY position.
    bgd_line: [(u8, bool); 160],

    /// 0xFF4F - VBK
    ///
    /// Indicates whether VRAM is switched to the 2nd bank.
    vram_banked: bool,

    /// The current mode the PPU is in.
    current_mode: PpuMode,

    /// Total cycles ticked under the current mode.
    total_cycles: u32,

    /// RGB24 framebuffer, this is the back buffer.
    back_framebuffer: Box<[u8; 160 * 144 * 3]>,

    /// RGB24 framebuffer, this is the front buffer.
    pub front_framebuffer: Box<[u8; 160 * 144 * 3]>,

    /// Shared reference to IF register.
    if_reg: Rc<RefCell<u8>>,
}

impl Ppu {
    /// Create a new `Ppu` instance.
    pub fn new(if_reg: Rc<RefCell<u8>>, cgb_mode: bool) -> Self {
        Self {
            vram: [0; 0x4000],
            oam_ram: [0; 0xA0],
            ly: 0,
            lyc: 0,
            lcdc: 0x91,
            stat: 0x00,
            scy: 0,
            scx: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wx: 0,
            wy: 0,
            window_line_counter: 0,
            cgb_mode,
            bcps: 0,
            bgd_palettes: [0; 0x40],
            ocps: 0,
            obj_palettes: [0; 0x40],
            bgd_line: [(0, false); 160],
            vram_banked: false,
            current_mode: PpuMode::OamSearch,
            total_cycles: 0,
            back_framebuffer: Box::new([0; 160 * 144 * 3]),
            front_framebuffer: Box::new([0; 160 * 144 * 3]),
            if_reg,
        }
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => {
                let offset = (addr - 0x8000) as usize;

                if self.cgb_mode && self.vram_banked {
                    self.vram[offset + 0x2000]
                } else {
                    self.vram[offset]
                }
            }

            0xFE00..=0xFE9F => self.oam_ram[(addr - 0xFE00) as usize],

            0xFF40 => self.lcdc,
            0xFF41 => (self.stat | 0x80) | self.current_mode as u8,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,

            0xFF4F => (self.vram_banked as u8) | 0xFE,
            0xFF68 => self.bcps | 0x40,
            0xFF69 => self.bgd_palettes[(self.bcps & 0x3F) as usize],
            0xFF6A => self.ocps | 0x40,
            0xFF6B => self.obj_palettes[(self.ocps & 0x3F) as usize],

            _ => unreachable!(),
        }
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => {
                let offset = (addr - 0x8000) as usize;

                if self.cgb_mode && self.vram_banked {
                    self.vram[offset + 0x2000] = value;
                } else {
                    self.vram[offset] = value;
                }
            }

            0xFE00..=0xFE9F => self.oam_ram[(addr - 0xFE00) as usize] = value,

            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = value & 0x78,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {}
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            0xFF4F => self.vram_banked = bit!(&value, 0),
            0xFF68 => self.bcps = value & 0xBF,
            0xFF69 => {
                let index = self.bcps & 0x3F;

                self.bgd_palettes[index as usize] = value;

                if bit!(&self.bcps, 7) {
                    self.bcps &= 0xC0;
                    self.bcps |= index.wrapping_add(1);
                }
            }
            0xFF6A => self.ocps = value & 0xBF,
            0xFF6B => {
                let index = self.ocps & 0x3F;

                self.obj_palettes[index as usize] = value;

                if bit!(&self.ocps, 7) {
                    self.ocps &= 0xC0;
                    self.ocps |= index.wrapping_add(1);
                }
            }

            _ => unreachable!(),
        }
    }

    /// Change the PPU's current mode.
    fn change_mode(&mut self, mode: PpuMode) {
        self.current_mode = mode;

        match mode {
            PpuMode::HBlank => {
                self.render_scanline();

                // Request STAT interrupt if HBlank bit
                // in LCD STAT is set.
                if bit!(&self.stat, 3) {
                    set!(self.if_reg.borrow_mut(), 1);
                }
            }

            PpuMode::VBlank => {
                // Request a VBlank interrupt.
                set!(self.if_reg.borrow_mut(), 0);

                // Request STAT interrupt if VBlank bit
                // in LCD STAT is set.
                if bit!(&self.stat, 4) {
                    set!(self.if_reg.borrow_mut(), 1);
                }
            }

            PpuMode::OamSearch => {
                // Request STAT interrupt if OamSearch bit
                // in LCD STAT is set.
                if bit!(&self.stat, 5) {
                    set!(self.if_reg.borrow_mut(), 1);
                }
            }

            _ => {}
        }
    }

    /// Compare LY and LYC and if they are equal,
    /// set coincidence bit in LCD STAT and request
    /// a STAT interrupt (if enabled).
    fn compare_lyc(&mut self) {
        if self.ly == self.lyc {
            set!(&mut self.stat, 2);

            if bit!(&self.stat, 6) {
                set!(self.if_reg.borrow_mut(), 1);
            }
        } else {
            res!(&mut self.stat, 2);
        }
    }

    /// Tick the PPU by 1 M cycle, and return a bool
    /// that tells if we have entered HBlank.
    pub fn tick(&mut self, cycles: u32) -> bool {
        if !bit!(&self.lcdc, 7) {
            return false;
        }

        self.total_cycles += cycles;

        let mut entered_hblank = false;

        // The actual PPU timings are not fixed.
        // They vary depending upon the number of sprites
        // on the screen, if the window is being drawn etc..
        match self.current_mode {
            PpuMode::OamSearch if self.total_cycles >= 80 => {
                self.total_cycles -= 80;
                self.change_mode(PpuMode::Drawing);
            }

            PpuMode::Drawing if self.total_cycles >= 172 => {
                self.total_cycles -= 172;
                self.change_mode(PpuMode::HBlank);

                if self.cgb_mode {
                    entered_hblank = true;
                }
            }

            PpuMode::HBlank if self.total_cycles >= 204 => {
                self.total_cycles -= 204;
                self.ly += 1;

                // LY 0x90 (144) signals end of one complete frame.
                if self.ly == 0x90 {
                    self.change_mode(PpuMode::VBlank);
                } else {
                    self.change_mode(PpuMode::OamSearch);
                }

                self.compare_lyc();
            }

            PpuMode::VBlank if self.total_cycles >= 456 => {
                self.total_cycles -= 456;
                self.ly += 1;

                // The PPU actually has 154 lines instead of 144.
                // These 10 lines are `psuedo lines` of sorts.
                if self.ly == 154 {
                    // Swap the copy the back buffer to the front buffer.
                    self.front_framebuffer
                        .copy_from_slice(self.back_framebuffer.as_ref());

                    self.ly = 0;
                    self.window_line_counter = 0;
                    self.change_mode(PpuMode::OamSearch);
                }

                self.compare_lyc();
            }

            _ => {}
        }

        entered_hblank
    }

    /// Render the current scanline.
    fn render_scanline(&mut self) {
        self.render_background();
        self.render_sprites();
    }

    /// Set a pixel in the framebuffer at the given `x` and `y`
    /// coordinates.
    fn set_pixel(&mut self, x: u8, y: u8, colour: u32) {
        let offset = (((y as usize) << 5) * 15) + (x as usize * 3);

        self.back_framebuffer[offset] = ((colour & 0xFF0000) >> 16) as u8;
        self.back_framebuffer[offset + 1] = ((colour & 0x00FF00) >> 8) as u8;
        self.back_framebuffer[offset + 2] = (colour & 0x0000FF) as u8;
    }

    /// Scale the CGB 5 bit RGB to standard 8 bit RGB.
    /// Colour Correction Algorithm taken from Byuu's (Near) blog.
    /// https://near.sh/articles/video/color-emulation
    fn scale_rgb(&self, cgb_colour: u16) -> u32 {
        let mut scaled = 0x000000;

        let red = (cgb_colour >> 0) & 0x1F;
        let green = (cgb_colour >> 5) & 0x1F;
        let blue = (cgb_colour >> 10) & 0x1F;

        let mut new_red = red * 26 + green * 4 + blue * 2;
        let mut new_green = green * 24 + blue * 8;
        let mut new_blue = red * 6 + green * 4 + blue * 22;

        new_red = new_red.min(960) >> 2;
        new_green = new_green.min(960) >> 2;
        new_blue = new_blue.min(960) >> 2;

        scaled |= (new_red as u32) << 16;
        scaled |= (new_green as u32) << 8;
        scaled |= new_blue as u32;

        scaled
    }

    /// Render the background map and the window map for this scanline.
    fn render_background(&mut self) {
        // The 0th bit of the LCDC in DMG mode when zero disables all forms
        // of background and window rendering.
        if !bit!(&self.lcdc, 0) && !self.cgb_mode {
            return;
        }

        // The window tile map that is to be rendered.
        let win_map = if bit!(&self.lcdc, 6) { 0x1C00 } else { 0x1800 };

        // The background tile map that is to be rendered.
        let bgd_map = if bit!(&self.lcdc, 3) { 0x1C00 } else { 0x1800 };

        // The tile data that is going to be used for rendering
        // the above tile maps.
        let tile_data = if bit!(&self.lcdc, 4) {
            0x0000u16
        } else {
            0x1000u16
        };

        // If the window is enabled this line, we increment
        // the internal line counter.
        let mut increment_window_counter = false;

        for x in 0u8..160u8 {
            // Extract the absolute X and Y coordinates of the pixel in
            // the respective 256 x 256 tile map.
            let (map_x, map_y, tile_map) =
                if bit!(&self.lcdc, 5) && self.wy <= self.ly && self.wx <= x + 7 {
                    let map_x = x.wrapping_add(7).wrapping_sub(self.wx);
                    let map_y = self.window_line_counter;

                    increment_window_counter = true;

                    (map_x, map_y, win_map)
                } else {
                    let map_x = x.wrapping_add(self.scx);
                    let map_y = self.ly.wrapping_add(self.scy);

                    (map_x, map_y, bgd_map)
                };

            // Extract the X and Y coordinates of the pixel inside the
            // respective tile.
            let mut tile_x = map_x & 0x07;
            let mut tile_y = map_y & 0x07;

            // Calculate the index for the tile number.
            let tile_number_index =
                tile_map + (((map_y as u16 >> 3) << 5) & 0x3FF) + ((map_x as u16 >> 3) & 0x1F);

            // Extract the tile number.
            let tile_number = self.vram[tile_number_index as usize];

            // Extract CGB background attributes.
            let cgb_bgd_attrs = self.vram[tile_number_index as usize + 0x2000];

            // If we are in CGB mode, check if we need to flip
            // the tile over the Y axis.
            if self.cgb_mode && bit!(&cgb_bgd_attrs, 6) {
                tile_y = 7 - tile_y;
            }

            // If we are in CGB mode, check if we need to flip
            // the tile over the X axis.
            if self.cgb_mode && !bit!(&cgb_bgd_attrs, 5) {
                tile_x = 7 - tile_x;
            }

            // Extract the address of the row we are rendering in the concerned tile.
            // There are two addressing modes,
            //
            // 1. 0x8000: (TILE_NUMBER as u8 * 16) + 0x8000.
            // 2. 0x8800: (TILE_NUMBER as i8 * 16) + 0x9000.
            let tile_address = if tile_data == 0x0000 {
                tile_data
                    .wrapping_add((tile_number as u16) << 4)
                    .wrapping_add((tile_y << 1) as u16)
            } else {
                tile_data
                    .wrapping_add(((tile_number as i8 as i16) as u16) << 4)
                    .wrapping_add((tile_y << 1) as u16)
            } as usize;

            if !self.cgb_mode {
                // Extract the colour data pertaining to the row.
                let lsb = self.vram[tile_address];
                let msb = self.vram[tile_address + 1];

                // Extract the colour pertaining to the pixel.
                let pixel_colour =
                    (((msb >> (7 - tile_x)) & 0x01) << 1) | ((lsb >> (7 - tile_x)) & 0x01);

                // Store the BG colour, and priority bit for later use.
                self.bgd_line[x as usize] = (pixel_colour, false);

                // Extract the actual pixel colour.
                let actual_pixel_colour =
                    DMG_MODE_PALETTE[((self.bgp >> (pixel_colour << 1)) & 0x03) as usize];

                self.set_pixel(x, self.ly, actual_pixel_colour);
            } else {
                // Extract the index of the colour palette we are
                // going to use to render the tile.
                let palette = (cgb_bgd_attrs & 0x07) as usize;

                // Extract which VRAM bank to take tile data from.
                let bank_offset = if bit!(&cgb_bgd_attrs, 3) {
                    0x2000
                } else {
                    0x0000
                };

                // Extract BG to OAM priority, and store it later for sprite
                // rendering.
                let bg_oam_priority = bit!(&cgb_bgd_attrs, 7);

                // Extract the colour data pertaining to the row.
                let lsb = self.vram[tile_address + bank_offset];
                let msb = self.vram[tile_address + bank_offset + 1];

                // Extract the colour pertaining to the pixel.
                let pixel_colour = (((msb >> tile_x) & 0x01) << 1) | ((lsb >> tile_x) & 0x01);

                // Store the BG colour, and priority bit for later use.
                self.bgd_line[x as usize] = (pixel_colour, bg_oam_priority);

                // The offset to the palette we are going to use.
                let palette_offset = (palette << 3) + ((pixel_colour as usize) << 1);

                // Extract the actual pixel colour.
                let actual_colour = ((self.bgd_palettes[palette_offset + 1] as u16) << 8)
                    | (self.bgd_palettes[palette_offset] as u16);

                self.set_pixel(x, self.ly, self.scale_rgb(actual_colour));
            }
        }

        self.window_line_counter += increment_window_counter as u8;
    }

    /// Render the sprites present on this scanline.
    fn render_sprites(&mut self) {
        // The 1st bit of LCDC controls whether OBJs (sprites)
        // are rendered or not.
        if !bit!(&self.lcdc, 1) {
            return;
        }

        // If the 2nd bit of LCDC is zero the sprite's size is taken to
        // be 8 x 8 else it's 8 x 16.
        let sprite_size = if bit!(&self.lcdc, 2) { 16 } else { 8 };

        // Go through OAM RAM and search for all sprites
        // that are visible in this scanline.
        // This is similar to what the PPU does in OAM search mode.
        //
        // The requirements for a sprite to be visible are,
        // 1. Y <= LY
        // 2. LY < (Y + SPRITE SIZE)
        let mut sprites = Vec::with_capacity(10);

        for i in (0x00..0xA0).step_by(4) {
            if sprites.len() == 10 {
                break;
            }

            let mut sprite: Sprite =
                unsafe { (*(self.oam_ram.as_ptr().add(i) as *const [u8; 4])).into() };

            sprite.y = sprite.y.wrapping_sub(16);
            sprite.x = sprite.x.wrapping_sub(8);

            if sprite_size == 16 {
                sprite.index &= 0xFE;
            }

            if (sprite.y <= self.ly) && (self.ly < sprite.y.wrapping_add(sprite_size)) {
                sprites.push(sprite);
            }
        }

        // Sort the sprites in a way that,
        //
        // 1. The sprite that has the lower X coordinate will draw
        //    over the sprite that has a higher X coordinate.
        // 2. The sprite that appeared earlier in the OAM RAM will draw
        //    over the sprite with same X coordinates.
        sprites.reverse();

        if !self.cgb_mode {
            sprites.sort_by(|&a, &b| b.x.cmp(&a.x));
        }

        for sprite in sprites {
            // Extract sprite attributes.
            let sprite_attr = sprite.flags;

            // Is the sprite flipped over the Y axis.
            let y_flip = bit!(&sprite_attr, 6);

            // Is the sprite flipped over the X axis.
            let x_flip = bit!(&sprite_attr, 5);

            // The palette used to render the sprite. (DMG)
            let palette = if bit!(&sprite_attr, 4) {
                self.obp1
            } else {
                self.obp0
            };

            // The palette used to render the sprite. (CGB)
            let colour_palette = (sprite_attr & 0x07) as usize;

            // The VRAM bank to use for getting the sprite tile in
            // CGB mode.
            let vram_offset = if self.cgb_mode && bit!(&sprite_attr, 3) {
                0x2000
            } else {
                0x0000
            };

            // Should the sprite be drawn over the background layer.
            // If this is false, the sprite will only be drawn
            // if the colour of BG is NOT 1-3.
            let sprite_over_bg = !bit!(&sprite_attr, 7);

            // The row in the tile of the sprite.
            let tile_y = if y_flip {
                sprite_size - (self.ly - sprite.y + 1)
            } else {
                self.ly - sprite.y
            };

            // The address of the sprite tile.
            let tile_address = (((sprite.index as u16) << 4) + ((tile_y as u16) << 1)) as usize;

            // Extract the colour data pertaining to the row.
            let lsb = self.vram[tile_address + vram_offset];
            let msb = self.vram[tile_address + vram_offset + 1];

            for x in 0..8 {
                let actual_x = sprite.x.wrapping_add(x);

                if actual_x < 160 {
                    // Get the index of the colour.
                    // 0 - Is always transparent for sprites.
                    let colour_index = if x_flip {
                        ((msb >> x & 0x01) << 1) | (lsb >> x & 0x01)
                    } else {
                        ((msb >> (7 - x) & 0x01) << 1) | (lsb >> (7 - x) & 0x01)
                    };

                    // Extract the actual RGB colour.
                    let colour = if self.cgb_mode {
                        let palette_offset = (colour_palette * 8) + (colour_index as usize * 2);

                        let cgb_colour = ((self.obj_palettes[palette_offset + 1] as u16) << 8)
                            | (self.obj_palettes[palette_offset] as u16);

                        self.scale_rgb(cgb_colour)
                    } else {
                        DMG_MODE_PALETTE[((palette >> (colour_index << 1)) & 0x03) as usize]
                    };

                    // We don't draw pixels that are transparent.
                    if colour_index != 0 {
                        if self.cgb_mode {
                            if !bit!(&self.lcdc, 0)
                                || (self.bgd_line[actual_x as usize].0 == 0)
                                || (!self.bgd_line[actual_x as usize].1 && sprite_over_bg)
                            {
                                self.set_pixel(actual_x, self.ly, colour);
                            }
                        } else if sprite_over_bg || self.bgd_line[actual_x as usize].0 == 0 {
                            self.set_pixel(actual_x, self.ly, colour);
                        }
                    }
                }
            }
        }
    }
}
