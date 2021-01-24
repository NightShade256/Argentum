use crate::common::MemInterface;

/// Trait implemented by all cartridges.
pub trait Cartridge: MemInterface {}

/// Cartridge with just two ROM banks.
/// Code: 0x00
pub struct RomOnly {
    memory: Box<[u8; 0x8000]>,
}

impl RomOnly {
    /// Create a new `RomOnly` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        assert_eq!(rom_buffer.len(), 0x8000);

        let mut memory = Box::new([0; 0x8000]);
        memory.copy_from_slice(rom_buffer);

        Self { memory }
    }
}

impl MemInterface for RomOnly {
    fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, _: u16, _: u8) {}
}

impl Cartridge for RomOnly {}
