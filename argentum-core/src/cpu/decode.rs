//! Contains functions to decode opcodes and dispatch
//! correct methods or perform the required work.

use super::*;

// WHOMSOEVER STRAY HITHER, READ AHEAD
//
// ACQUIRE A LOOKALIKE OF THIS SACRED TEXT
// https://cdn.discordapp.com/attachments/465586075830845475/742438340078469150/SM83_decoding.pdf
// THEN THOU MIGHT BEGIN TO UNDERSTAND THE GIBBERISH BENEATH.

/// R16 (group 1)
const R16_GROUP_ONE: [Reg16; 4] = [Reg16::BC, Reg16::DE, Reg16::HL, Reg16::SP];

/// R16 (group 2)
const R16_GROUP_TWO: [Reg16; 4] = [Reg16::BC, Reg16::DE, Reg16::HLI, Reg16::HLD];

impl Cpu {
    pub fn decode_and_execute(&mut self, bus: &mut Bus, opcode: u16) {
        match opcode {
            0x00 => self.nop(),
            0x08 => self.ld_u16_sp(bus),
            0x10 => self.stop(bus),
            0x18 => self.jr(bus),

            0x20 | 0x28 | 0x30 | 0x38 => {
                self.conditional_jr(bus, ((opcode >> 3) & 0x3) as u8);
            }

            0x01 | 0x11 | 0x21 | 0x31 => {
                let reg = R16_GROUP_ONE[((opcode >> 4) & 0x3) as usize];
                let value = self.imm_word(bus);

                self.r.write_rr(reg, value);
            }

            0x09 | 0x19 | 0x29 | 0x39 => {
                let reg = R16_GROUP_ONE[((opcode >> 4) & 0x3) as usize];
                self.add_hl_rr(reg);
            }

            0x02 | 0x12 | 0x22 | 0x32 => {
                let reg = R16_GROUP_TWO[((opcode >> 4) & 0x3) as usize];
                let addr = self.r.read_rr(reg);

                bus.write_byte(addr, self.r.a);
            }

            0x0A | 0x1A | 0x2A | 0x3A => {
                let reg = R16_GROUP_TWO[((opcode >> 4) & 0x3) as usize];
                let addr = self.r.read_rr(reg);

                self.r.a = bus.read_byte(addr);
            }

            0x03 | 0x13 | 0x23 | 0x33 => {
                let reg = R16_GROUP_ONE[((opcode >> 4) & 0x3) as usize];
                let value = self.r.read_rr(reg);

                self.r.write_rr(reg, value.wrapping_add(1));
            }

            0x0B | 0x1B | 0x2B | 0x3B => {
                let reg = R16_GROUP_ONE[((opcode >> 4) & 0x3) as usize];
                let value = self.r.read_rr(reg);

                self.r.write_rr(reg, value.wrapping_sub(1));
            }

            0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => {
                let r8 = unsafe { std::mem::transmute((opcode >> 3) as u8) };
                self.inc_r8(bus, r8);
            }

            0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => {
                let r8 = unsafe { std::mem::transmute((opcode >> 3) as u8) };
                self.dec_r8(bus, r8);
            }

            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                let r8 = unsafe { std::mem::transmute((opcode >> 3) as u8) };
                let value = self.imm_byte(bus);

                self.r.set_r8(r8, bus, value);
            }

            // HALT (will implement later).
            0x76 => {}

            0x40..=0x75 => {
                let src = unsafe { std::mem::transmute((opcode & 0x7) as u8) };
                let dest = unsafe { std::mem::transmute((opcode >> 3) as u8) };

                let value = self.r.get_r8(src, bus);
                self.r.set_r8(dest, bus, value);
            }

            0x77..=0x7F => {
                let src = unsafe { std::mem::transmute((opcode & 0x7) as u8) };
                let dest = unsafe { std::mem::transmute((opcode >> 3) as u8) };

                let value = self.r.get_r8(src, bus);
                self.r.set_r8(dest, bus, value);
            }

            _ => {}
        }
    }
}
