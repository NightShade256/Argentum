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

/// R16 (group 3)
const R16_GROUP_THR: [Reg16; 4] = [Reg16::BC, Reg16::DE, Reg16::HL, Reg16::AF];

const OPCODE_GROUP_ONE: [fn(&mut Cpu, &mut Bus, Reg8) -> (); 8] = [
    Cpu::add_r8,
    Cpu::adc_r8,
    Cpu::sub_r8,
    Cpu::sbc_r8,
    Cpu::and_r8,
    Cpu::xor_r8,
    Cpu::or_r8,
    Cpu::cp_r8,
];

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

            0x80..=0xBF => {
                let reg = unsafe { std::mem::transmute((opcode & 0x7) as u8) };
                let index = (opcode >> 3) as u8;

                // Should I use match with unreachable macro?
                OPCODE_GROUP_ONE[index as usize](self, bus, reg);
            }

            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                self.conditional_ret(bus, ((opcode >> 3) & 0x3) as u8);
            }

            0xE0 => {
                let addr = (0xFF00u16).wrapping_add(self.imm_byte(bus) as u16);
                bus.write_byte(addr, self.r.a);
            }

            0xE8 => {
                let offset = self.imm_byte(bus) as i8 as i16;
                let sp = self.r.sp;

                self.r.sp = (sp as i16 + offset) as u16;

                self.r.set_zf(false);
                self.r.set_nf(false);
                self.r
                    .set_hf(((sp & 0xF) + (offset as u16 & 0xF)) & 0x10 == 0x10);
                self.r.set_cf(sp + offset as u16 > 0xFF);
            }

            0xF0 => {
                let addr = (0xFF00u16).wrapping_add(self.imm_byte(bus) as u16);
                self.r.a = bus.read_byte(addr);
            }

            0xF8 => {
                let offset = self.imm_byte(bus) as i8 as i16;
                let sp = self.r.sp;

                self.r.write_rr(Reg16::HL, (sp as i16 + offset) as u16);

                self.r.set_zf(false);
                self.r.set_nf(false);
                self.r
                    .set_hf(((sp & 0xF) + (offset as u16 & 0xF)) & 0x10 == 0x10);
                self.r.set_cf(sp + offset as u16 > 0xFF);
            }

            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let reg = R16_GROUP_THR[((opcode >> 4) & 0x3) as usize];
                let pop = self.pop(bus);

                self.r.write_rr(reg, pop);
            }

            0xC9 => self.ret(bus),
            0xD9 => {
                self.ret(bus);
                self.ime = true;
            }
            0xE9 => {
                let addr = self.r.read_rr(Reg16::HL);
                self.jp(addr);
            }
            0xF9 => self.r.sp = self.r.read_rr(Reg16::HL),

            _ => {}
        }
    }
}
