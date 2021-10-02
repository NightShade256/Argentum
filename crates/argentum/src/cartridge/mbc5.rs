use super::*;

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

    fn get_sram(&self) -> Option<Vec<u8>> {
        if !self.ram.is_empty() {
            Some(self.ram.clone())
        } else {
            None
        }
    }
}
