use super::Bus;
use crate::helpers::BitExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferType {
    Gdma,
    Hdma,
}

#[derive(Default)]
pub struct CgbDma {
    control: u8,
    dst: u16,
    len: u16,
    src: u16,
    status: Option<TransferType>,
}

impl CgbDma {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF51..=0xFF54 => 0xFF,
            0xFF55 => self.control,

            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF51 => {
                self.src &= 0x00FF;
                self.src |= (value as u16) << 8;
            }

            0xFF52 => {
                self.src &= 0xFF00;
                self.src |= (value as u16) & 0xF0;
            }

            0xFF53 => {
                self.dst &= 0x00FF;
                self.dst |= (value as u16) << 8;
            }

            0xFF54 => {
                self.dst &= 0xFF00;
                self.dst |= (value as u16) & 0xF0;
            }

            0xFF55 => {
                self.control = value;
                self.len = (((value & 0x7F) as u16) + 1) << 4;

                if value.bit(7) {
                    self.status = Some(TransferType::Hdma);
                } else {
                    if let Some(TransferType::Hdma) = self.status {
                        self.control = 0xFF;
                        self.status = None;
                    } else {
                        self.status = Some(TransferType::Gdma);
                    }
                }
            }

            _ => unreachable!(),
        }
    }
}

impl Bus {
    pub fn tick_cgb_dma(&mut self, hblank: bool) {
        if let Some(transfer_type) = self.cgb_dma.status {
            if transfer_type == TransferType::Hdma && hblank {
                for offset in 0..0x10 {
                    let value = self.read_byte(self.cgb_dma.src + offset, false);

                    self.ppu
                        .write_byte(((self.cgb_dma.dst + offset) & 0x1FFF) + 0x8000, value);
                }

                self.cgb_dma.len -= 0x10;
                self.cgb_dma.src += 0x10;
                self.cgb_dma.dst += 0x10;

                self.cgb_dma.control -= 1;

                if self.cgb_dma.len == 0 {
                    self.cgb_dma.control = 0xFF;
                    self.cgb_dma.status = None;
                }
            }

            if transfer_type == TransferType::Gdma {
                for offset in 0..self.cgb_dma.len {
                    let value = self.read_byte(self.cgb_dma.src + offset, false);
                    self.write_byte(self.cgb_dma.dst + offset, value, false);
                }

                self.cgb_dma.control = 0xFF;
                self.cgb_dma.status = None;
            }
        }
    }
}
