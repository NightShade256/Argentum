use crate::common::MemInterface;

// Todo
//
// 1. MBC1 and MBC5 (maybe MBC2?)
// 2. Save Files for Battery Buffered RAM (unsure how to structure API).

/// RAM Size corresponding to indices
/// in cartridge headers.
/// Only contains till 32 KBytes
/// since we only implement MBC3.
const RAM_SIZES: [usize; 4] = [0x0000, 0x0500, 0x2000, 0x8000];

/// Trait implemented by all cartridges.
pub trait Cartridge: MemInterface {
    /// Return the title of the game.
    fn game_title(&self) -> String;
}

/// Cartridge with just two ROM banks.
pub struct RomOnly {
    /// Two ROM banks each of 4KB.
    memory: Box<[u8; 0x8000]>,
}

impl RomOnly {
    /// Create a new `RomOnly` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        assert_eq!(rom_buffer.len(), 0x8000);

        // Load the cartridge into memory.
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

impl Cartridge for RomOnly {
    fn game_title(&self) -> String {
        String::from_utf8_lossy(&self.memory[0x134..0x0143]).into()
    }
}

/// Cartridge with the MBC3 chip.
/// Max 2MB ROM and 32KB RAM.
pub struct Mbc3 {
    /// ROM with a maximum size of 2 MB.
    memory: Vec<u8>,

    /// The ROM bank that is currently mapped
    /// to the addresses 0x4000 to 0x7FFF.
    rom_bank: usize,

    /// External RAM with a maximum size of 32 KB.
    external_ram: Vec<u8>,

    /// Is the external RAM enabled currently enabled?
    external_ram_enabled: bool,

    /// The current external RAM bank that is mapped
    /// to the addresses 0xA000 to 0xBFFF.
    ram_bank: usize,
}

impl Mbc3 {
    /// Create a new `Mbc3` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        // Load ROM into memory.
        let mut memory = vec![0u8; rom_buffer.len()];
        memory.copy_from_slice(rom_buffer);

        // Initialize external ram if any.
        let external_ram = vec![0u8; RAM_SIZES[rom_buffer[0x0149] as usize]];

        Self {
            memory,
            rom_bank: 1,
            external_ram,
            external_ram_enabled: false,
            ram_bank: 0,
        }
    }
}

impl MemInterface for Mbc3 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Only ROM bank 0 is ever mapped to this range.
            0x0000..=0x3FFF => self.memory[addr as usize],

            // Each bank is 0x4000 in length.
            0x4000..=0x7FFF => self.memory[(self.rom_bank * 0x4000) + (addr - 0x4000) as usize],

            // We only read valid data if the external RAM was previously enabled.
            0xA000..=0xBFFF if self.external_ram_enabled => {
                self.external_ram[(self.ram_bank * 0x2000) + (addr - 0xA000) as usize]
            }

            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // External RAM enable register.
            0x0000..=0x1FFF => {
                self.external_ram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM bank register.
            // Bank 1 is implicitly selected on a 0x00 write.
            // Since Bank 0 is always mapped to 0x0000 - 0x3FFF.
            0x2000..=0x3FFF => {
                self.rom_bank = if value == 0 { 1 } else { value } as usize;
            }

            // Not going to emulate RTC.
            0x4000..=0x5FFF if value <= 0x03 => {
                self.ram_bank = value as usize;
            }

            // We only write data if the external RAM was previously enabled.
            0xA000..=0xBFFF if self.external_ram_enabled => {
                self.external_ram[(self.ram_bank * 0x2000) + (addr - 0xA000) as usize] = value;
            }

            _ => {}
        }
    }
}

impl Cartridge for Mbc3 {
    fn game_title(&self) -> String {
        String::from_utf8_lossy(&self.memory[0x134..0x0143]).into()
    }
}
