use super::CPU;
use crate::bus::Bus;

impl CPU {
    pub fn decode_and_execute(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            0x00 => self.nop(),

            0x08 => self.ld_u16_sp(bus),

            0x10 => self.stop(),

            0x18 => self.unconditional_jr(bus),

            0x20 | 0x28 | 0x30 | 0x38 => {
                let condition = (opcode >> 3) & 0x3;

                self.conditional_jr(bus, condition);
            }

            _ => log::warn!(
                "Invalid operation code {:#04X} encountered at PC={:#06X}.",
                opcode,
                self.r.pc
            ),
        }
    }
}
