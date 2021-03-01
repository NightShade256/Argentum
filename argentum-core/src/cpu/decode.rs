use super::CPU;
use crate::bus::Bus;

impl CPU {
    pub fn decode_and_execute(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            0x00 => self.nop(),

            _ => log::warn!(
                "Invalid operation code {:#04X} encountered at PC={:#06X}.",
                opcode,
                self.r.pc
            ),
        }
    }
}
