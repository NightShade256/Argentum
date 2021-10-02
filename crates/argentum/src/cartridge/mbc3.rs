use super::*;

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
    pub fn new(rom: &[u8], save_file: Option<Vec<u8>>) -> Self {
        let mut ram = vec![0u8; RAM_SIZES[rom[0x0149] as usize]];

        if !ram.is_empty() {
            if let Some(ram_save) = save_file {
                if ram.len() == ram_save.len() {
                    ram.copy_from_slice(&ram_save);
                }
            }
        }

        Self {
            rom: rom.to_vec(),
            ram,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banks: 2 * 2usize.pow(rom[0x0148] as u32),
            ram_banks: (RAM_SIZES[rom[0x0149] as usize] >> 13) as usize,
        }
    }
}

impl Cartridge for Mbc3 {
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

    fn get_sram(&self) -> Option<Vec<u8>> {
        if !self.ram.is_empty() {
            Some(self.ram.clone())
        } else {
            None
        }
    }
}
