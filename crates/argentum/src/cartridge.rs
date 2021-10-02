use crate::helpers::BitExt;

mod mbc0;
mod mbc1;
mod mbc3;
mod mbc5;

/// RAM Size corresponding to indices
/// in cartridge headers.
const RAM_SIZES: [usize; 6] = [0x0000, 0x0000, 0x2000, 0x8000, 0x20000, 0x10000];

/// Trait implemented by all cartridges.
pub trait Cartridge {
    /// Read a byte from the given address.
    fn read_byte(&self, addr: u16) -> u8;

    /// Write a byte to the given address.
    fn write_byte(&mut self, addr: u16, value: u8);

    /// Get SRAM as a vector of bytes if present.
    fn get_sram(&self) -> Option<Vec<u8>>;

    /// Returns `true` if the game is CGB compatible.
    fn is_cgb_compatible(&self) -> bool {
        self.read_byte(0x0143).bit(7)
    }
}

pub fn make_cartridge(rom: Vec<u8>, save_file: Option<Vec<u8>>) -> Box<dyn Cartridge> {
    match rom[0x0147] {
        0x00 => Box::new(mbc0::Mbc0::new(&rom)),
        0x01..=0x03 => Box::new(mbc1::Mbc1::new(&rom)),
        0x0F..=0x13 => Box::new(mbc3::Mbc3::new(&rom, save_file)),
        0x19..=0x1E => Box::new(mbc5::Mbc5::new(&rom)),

        _ => panic!("unsupported cartridge type"),
    }
}
