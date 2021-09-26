use super::Bus;
use crate::helpers::bit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferType {
    Gdma,
    Hdma,
}

#[derive(Default)]
pub struct DmaController {
    dma_control: u8,
    dma_dst: u16,
    dma_len: u16,
    dma_src: u16,
    transfer: Option<TransferType>,
}

impl DmaController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF51..=0xFF54 => 0xFF,
            0xFF55 => self.dma_control,

            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF51 => {
                self.dma_src &= 0x00FF;
                self.dma_src |= (value as u16) << 8;
            }

            0xFF52 => {
                self.dma_src &= 0xFF00;
                self.dma_src |= (value as u16) & 0xF0;
            }

            0xFF53 => {
                self.dma_dst &= 0x00FF;
                self.dma_dst |= (value as u16) << 8;
            }

            0xFF54 => {
                self.dma_dst &= 0xFF00;
                self.dma_dst |= (value as u16) & 0xF0;
            }

            0xFF55 => {
                self.dma_control = value;
                self.dma_len = (((value & 0x7F) as u16) + 1) << 4;

                if bit!(&value, 7) {
                    self.transfer = Some(TransferType::Hdma);
                } else {
                    if let Some(TransferType::Hdma) = self.transfer {
                        self.dma_control = 0xFF;
                        self.transfer = None;
                    } else {
                        self.transfer = Some(TransferType::Gdma);
                    }
                }
            }

            _ => unreachable!(),
        }
    }
}

impl Bus {
    pub fn tick_dma_controller(&mut self, hblank: bool) {
        if let Some(transfer_type) = self.dma.transfer {
            if transfer_type == TransferType::Hdma && hblank {
                for offset in 0..0x10 {
                    let value = self.read_byte(self.dma.dma_src + offset, false);

                    self.ppu
                        .write_byte(((self.dma.dma_dst + offset) & 0x1FFF) + 0x8000, value);
                }

                self.dma.dma_len -= 0x10;
                self.dma.dma_src += 0x10;
                self.dma.dma_dst += 0x10;

                self.dma.dma_control -= 1;

                if self.dma.dma_len == 0 {
                    self.dma.dma_control = 0xFF;
                    self.dma.transfer = None;
                }
            }

            if transfer_type == TransferType::Gdma {
                for offset in 0..self.dma.dma_len {
                    let value = self.read_byte(self.dma.dma_src + offset, false);
                    self.write_byte(self.dma.dma_dst + offset, value, false);
                }

                self.dma.dma_control = 0xFF;
                self.dma.transfer = None;
            }
        }
    }
}
