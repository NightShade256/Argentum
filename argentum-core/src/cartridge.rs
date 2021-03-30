use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// RAM Size corresponding to indices
/// in cartridge headers.
const RAM_SIZES: [usize; 6] = [0x0000, 0x0000, 0x2000, 0x8000, 0x20000, 0x10000];

/// Trait implemented by all cartridges.
pub trait Cartridge {
    /// Return the title of the game.
    fn game_title(&self) -> String;

    /// Read a byte from the given address.
    fn read_byte(&self, addr: u16) -> u8;

    /// Write a byte to the given address.
    fn write_byte(&mut self, addr: u16, value: u8);

    /// Detects whether the game is a CGB game.
    fn has_cgb_support(&self) -> bool {
        let cgb_flag_byte = self.read_byte(0x0143);

        // A game supports CGB functions if the upper bit is set.
        (cgb_flag_byte & 0x80) != 0
    }
}

/// Cartridge with just two ROM banks.
pub struct RomOnly {
    /// Two ROM banks each of 4KB.
    rom: Vec<u8>,
}

impl RomOnly {
    /// Create a new `RomOnly` instance.
    pub fn new(rom: &[u8]) -> Self {
        Self { rom: rom.to_vec() }
    }
}

impl Cartridge for RomOnly {
    fn game_title(&self) -> String {
        if self.has_cgb_support() {
            String::from_utf8_lossy(&self.rom[0x134..=0x013E]).into()
        } else {
            String::from_utf8_lossy(&self.rom[0x134..=0x0143]).into()
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write_byte(&mut self, _: u16, _: u8) {}
}

/// Cartridge with the MBC1 chip.
/// Max 16 MBit ROM and 256 KBit RAM.
pub struct Mbc1 {
    /// ROM with a maximum size of 16 MBit.
    rom: Vec<u8>,

    /// RAM with a maximum size of 256Kbit.
    ram: Vec<u8>,

    /// RAM gate register.
    /// Used to enable access to the external RAM.
    ram_enabled: bool,

    /// ROM bank register (lower).
    /// Stores the lower 5 bits of the ROM bank.
    /// The lower 5 bits cannot contain an zero bit pattern.
    rom_bank_lower: u8,

    /// ROM bank register (upper).
    /// Stores the upper 2 bits of the ROM bank.
    rom_bank_upper: u8,

    /// The banking mode currently in use.
    banking_mode: bool,

    /// The number of ROM banks in the cartridge.
    rom_banks: usize,

    /// The number of RAM banks in the cartridge.
    ram_banks: usize,
}

impl Mbc1 {
    /// Create a new `Mbc1` instance.
    pub fn new(rom: &[u8]) -> Self {
        Self {
            rom: rom.to_vec(),
            ram: vec![0u8; RAM_SIZES[rom[0x0149] as usize]],
            ram_enabled: false,
            rom_bank_lower: 1,
            rom_bank_upper: 0,
            banking_mode: false,
            rom_banks: 2 * 2usize.pow(rom[0x0148] as u32),
            ram_banks: (RAM_SIZES[rom[0x0149] as usize] >> 13) as usize,
        }
    }
}

impl Cartridge for Mbc1 {
    fn game_title(&self) -> String {
        if self.has_cgb_support() {
            String::from_utf8_lossy(&self.rom[0x134..=0x013E]).into()
        } else {
            String::from_utf8_lossy(&self.rom[0x134..=0x0143]).into()
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                let bank = if self.banking_mode {
                    (self.rom_bank_upper as usize) << 5
                } else {
                    0
                } % self.rom_banks;

                let addr = (bank * 0x4000) + addr as usize;

                self.rom[addr]
            }

            0x4000..=0x7FFF => {
                let bank = ((self.rom_bank_lower as usize) | ((self.rom_bank_upper as usize) << 5))
                    % self.rom_banks;

                let addr = (bank * 0x4000) + (addr as usize - 0x4000);

                self.rom[addr]
            }

            0xA000..=0xBFFF if self.ram_enabled => {
                let bank = if self.banking_mode {
                    self.rom_bank_upper as usize
                } else {
                    0
                } % self.ram_banks;

                let addr = (bank * 0x2000) + (addr as usize - 0xA000);

                self.ram[addr]
            }

            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0b1010;
            }

            0x2000..=0x3FFF => {
                if (value & 0b11111) == 0 {
                    self.rom_bank_lower = 1;
                } else {
                    self.rom_bank_lower = value & 0b11111;
                }
            }

            0x4000..=0x5FFF => {
                self.rom_bank_upper = value & 0b11;
            }

            0x6000..=0x7FFF => {
                self.banking_mode = (value & 0b1) != 0;
            }

            0xA000..=0xBFFF if self.ram_enabled => {
                let bank = if self.banking_mode {
                    self.rom_bank_upper as usize
                } else {
                    0
                } % self.ram_banks;

                let addr = (bank * 0x2000) + (addr as usize - 0xA000);

                self.ram[addr] = value;
            }

            _ => log::warn!("Write to external RAM occurred without first enabling it."),
        }
    }
}

/// Cartridge with the MBC3 chip.
/// Max 16 Mbit ROM and 256 KBit RAM.
pub struct Mbc3 {
    /// ROM with a maximum size of 16 MBit.
    rom: Vec<u8>,

    /// RAM with a maximum size of 256 Kbit.
    ram: Vec<u8>,

    /// RAM gate register.
    /// Used to enable access to the external RAM and timer.
    ram_enabled: bool,

    /// ROM Bank register.
    /// Used to store the selected ROM bank.
    rom_bank: u8,

    /// RAMB register.
    /// Used to store the 4 bits of the RAM bank in use.
    ram_bank: u8,

    /// The number of ROM banks in the cartridge.
    rom_banks: usize,

    /// The number of RAM banks in the cartridge.
    ram_banks: usize,
}

impl Mbc3 {
    /// Create a new `Mbc3` instance.
    pub fn new(rom: &[u8]) -> Self {
        Self {
            rom: rom.to_vec(),
            ram: vec![0u8; RAM_SIZES[rom[0x0149] as usize]],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banks: 2 * 2usize.pow(rom[0x0148] as u32),
            ram_banks: (RAM_SIZES[rom[0x0149] as usize] >> 13) as usize,
        }
    }
}

impl Cartridge for Mbc3 {
    fn game_title(&self) -> String {
        if self.has_cgb_support() {
            String::from_utf8_lossy(&self.rom[0x134..=0x013E]).into()
        } else {
            String::from_utf8_lossy(&self.rom[0x134..=0x0143]).into()
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],

            0x4000..=0x7FFF => {
                let bank = self.rom_bank as usize % self.rom_banks;

                let addr = (bank * 0x4000) + (addr as usize - 0x4000);

                self.rom[addr]
            }

            0xA000..=0xBFFF if self.ram_enabled => {
                let addr =
                    (0x2000 * (self.ram_bank as usize % self.ram_banks)) + (addr as usize - 0xA000);

                self.ram[addr]
            }

            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0b1010;
            }

            0x2000..=0x3FFF => {
                self.rom_bank = if (value & 0b0111_1111) == 0 {
                    1
                } else {
                    value & 0b0111_1111
                };
            }

            0x4000..=0x5FFF => {
                self.ram_bank = value & 0b11;
            }

            0xA000..=0xBFFF if self.ram_enabled => {
                let addr =
                    (0x2000 * (self.ram_bank as usize % self.ram_banks)) + (addr as usize - 0xA000);

                self.ram[addr] = value;
            }

            _ => {}
        }
    }
}

/// Cartridge with the MBC5 chip.
/// Max 64 Mbit ROM and 1 MBit RAM.
pub struct Mbc5 {
    /// ROM with a maximum size of 64 MBit.
    rom: Vec<u8>,

    /// RAM with a maximum size of 1 Mbit.
    ram: Vec<u8>,

    /// RAM gate register.
    /// Used to enable access to the external RAM.
    ram_enabled: bool,

    /// ROMB0 register.
    /// Used to store the lower 8 bits of the ROM bank.
    rom_bank_lower: u8,

    /// ROMB1 register.
    /// Used to store the upper 1 bit of the ROM bank.
    rom_bank_upper: u8,

    /// RAMB register.
    /// Used to store the 4 bits of the RAM bank in use.
    ram_bank: u8,

    /// The number of ROM banks in the cartridge.
    rom_banks: usize,

    /// The number of RAM banks in the cartridge.
    ram_banks: usize,
}

impl Mbc5 {
    /// Create a new `Mbc5` instance.
    pub fn new(rom: &[u8]) -> Self {
        Self {
            rom: rom.to_vec(),
            ram: vec![0u8; RAM_SIZES[rom[0x0149] as usize]],
            ram_enabled: false,
            rom_bank_lower: 1,
            rom_bank_upper: 0,
            ram_bank: 0,
            rom_banks: 2 * 2usize.pow(rom[0x0148] as u32),
            ram_banks: (RAM_SIZES[rom[0x0149] as usize] >> 13) as usize,
        }
    }
}

impl Cartridge for Mbc5 {
    fn game_title(&self) -> String {
        if self.has_cgb_support() {
            String::from_utf8_lossy(&self.rom[0x134..=0x013E]).into()
        } else {
            String::from_utf8_lossy(&self.rom[0x134..=0x0143]).into()
        }
    }

    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],

            0x4000..=0x7FFF => {
                let mut bank =
                    ((self.rom_bank_upper as usize) << 8) | (self.rom_bank_lower as usize);

                bank %= self.rom_banks;

                let addr = (bank * 0x4000) + (addr as usize - 0x4000);

                self.rom[addr]
            }

            0xA000..=0xBFFF if self.ram_enabled => {
                let addr =
                    (0x2000 * (self.ram_bank as usize % self.ram_banks)) + (addr as usize - 0xA000);

                self.ram[addr]
            }

            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = value == 0b0000_1010;
            }

            0x2000..=0x2FFF => {
                self.rom_bank_lower = value;
            }

            0x3000..=0x3FFF => {
                self.rom_bank_upper = value & 0b1;
            }

            0x4000..=0x5FFF => {
                self.ram_bank = value & 0b1111;
            }

            0xA000..=0xBFFF if self.ram_enabled => {
                let addr = (0x2000 * self.ram_bank as usize) + (addr as usize - 0xA000);

                self.ram[addr] = value;
            }

            _ => {}
        }
    }
}
