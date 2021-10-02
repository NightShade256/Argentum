use super::*;

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

            _ => {}
        }
    }

    fn get_sram(&self) -> Option<Vec<u8>> {
        if !self.ram.is_empty() {
            Some(self.ram.clone())
        } else {
            None
        }
    }
}
