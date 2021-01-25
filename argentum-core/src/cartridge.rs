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
    fn game_title(&self) -> &str;
}

/// Cartridge with just two ROM banks.
/// Code: 0x00
pub struct RomOnly {
    /// Two ROM banks each of 4KB.
    memory: Box<[u8; 0x8000]>,

    /// The title of the game.
    title: String,
}

impl RomOnly {
    /// Create a new `RomOnly` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        assert_eq!(rom_buffer.len(), 0x8000);

        // Load the cartridge into memory.
        let mut memory = Box::new([0; 0x8000]);
        memory.copy_from_slice(rom_buffer);

        // Extract the game title.
        let title_bytes = rom_buffer[0x0134..=0x143].to_vec();
        let title = String::from_utf8(title_bytes).unwrap();

        Self { memory, title }
    }
}

impl MemInterface for RomOnly {
    fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, _: u16, _: u8) {}
}

impl Cartridge for RomOnly {
    fn game_title(&self) -> &str {
        self.title.as_str()
    }
}

/// Probably the most sloppy implementation of MBC3.
pub struct Mbc3 {
    /// ROM of variable length.
    memory: Vec<u8>,

    /// The current ROM bank that is mapped.
    rom_bank: usize,

    /// External RAM, if any.
    eram: Vec<u8>,

    /// Is ERAM enabled?
    eram_enabled: bool,

    /// The current ERAM bank that is mapped.
    ram_bank: usize,

    /// The title of the game.
    title: String,
}

impl Mbc3 {
    pub fn new(rom_buffer: &[u8]) -> Self {
        // Load ROM into memory.
        let mut memory = vec![0u8; rom_buffer.len()];
        memory.copy_from_slice(rom_buffer);

        // Extract the game title.
        let title_bytes = rom_buffer[0x0134..=0x143].to_vec();
        let title = String::from_utf8(title_bytes).unwrap();

        // Init ERAM, if any.
        let eram_size = RAM_SIZES[rom_buffer[0x0149] as usize];
        let eram = vec![0u8; eram_size];

        Self {
            memory,
            eram,
            eram_enabled: false,
            title,
            rom_bank: 1,
            ram_bank: 0,
        }
    }
}

impl MemInterface for Mbc3 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM
            0x0000..=0x3FFF => self.memory[addr as usize],
            0x4000..=0x7FFF => self.memory[(self.rom_bank * 0x4000) + (addr - 0x4000) as usize],

            // ERAM
            0xA000..=0xBFFF if self.eram_enabled => {
                self.eram[(self.ram_bank * 0x2000) + (addr - 0xA000) as usize]
            }

            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // ERAM enable register.
            0x0000..=0x1FFF => {
                self.eram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM bank register.
            // Bank 1 is implicitly selected on a 0x00 write.
            // Since Bank 0 is always mapped to 0x0000 - 0x3FFF
            0x2000..=0x3FFF => {
                self.rom_bank = if value == 0 { 1 } else { value } as usize;
            }

            // Not going to emulate RTC.
            0x4000..=0x5FFF if value <= 0x03 => {
                self.ram_bank = value as usize;
            }

            // ERAM
            0xA000..=0xBFFF if self.eram_enabled => {
                self.eram[(self.ram_bank * 0x2000) + (addr - 0xA000) as usize] = value;
            }

            _ => {}
        }
    }
}

impl Cartridge for Mbc3 {
    fn game_title(&self) -> &str {
        self.title.as_str()
    }
}
