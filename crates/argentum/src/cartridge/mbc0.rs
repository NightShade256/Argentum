use super::*;

/// Cartridge with just two ROM banks.
pub struct Mbc0 {
    /// Two ROM banks each of 4KB.
    rom: Vec<u8>,
}

impl Mbc0 {
    /// Create a new `RomOnly` instance.
    pub fn new(rom: &[u8]) -> Self {
        Self { rom: rom.to_vec() }
    }
}

impl Cartridge for Mbc0 {
    fn read_byte(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write_byte(&mut self, _: u16, _: u8) {
        /* writes are ignored when there is no MBC */
    }

    fn get_sram(&self) -> Option<Vec<u8>> {
        None
    }
}
