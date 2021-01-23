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

/// Special OpCodes group.
const OPCODE_GROUP_ONE: [fn(&mut Cpu) -> (); 8] = [
    Cpu::rlca,
    Cpu::rrca,
    Cpu::rla,
    Cpu::rra,
    Cpu::daa,
    Cpu::cpl,
    Cpu::scf,
    Cpu::ccf,
];

/// ALU OpCodes group.
const OPCODE_GROUP_TWO: [fn(&mut Cpu, u8) -> (); 8] = [
    Cpu::add_r8,
    Cpu::adc_r8,
    Cpu::sub_r8,
    Cpu::sbc_r8,
    Cpu::and_r8,
    Cpu::xor_r8,
    Cpu::or_r8,
    Cpu::cp_r8,
];

// CB shift/rotate OpCodes group.
const OPCODE_GROUP_THR: [fn(&mut Cpu, &mut Bus, Reg8) -> (); 8] = [
    Cpu::rlc_r8,
    Cpu::rrc_r8,
    Cpu::rl_r8,
    Cpu::rr_r8,
    Cpu::sla_r8,
    Cpu::sra_r8,
    Cpu::swap_r8,
    Cpu::srl_r8,
];

impl Cpu {
    pub fn decode_and_execute(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            // NOP
            0x00 => self.nop(),

            // LD [u16], SP
            0x08 => self.ld_u16_sp(bus),

            // STOP
            0x10 => self.stop(bus),

            // JR (unconditional)
            0x18 => self.jr(bus),

            // JR (conditional)
            0x20 | 0x28 | 0x30 | 0x38 => {
                let condition = (opcode >> 3) & 0x3;
                self.conditional_jr(bus, condition);
            }

            // LD r16, u16
            0x01 | 0x11 | 0x21 | 0x31 => {
                let index = ((opcode >> 4) & 0x3) as usize;

                let r16 = R16_GROUP_ONE[index];
                let imm = self.imm_word(bus);

                self.r.write_r16(r16, imm);
            }

            // ADD HL, r16
            0x09 | 0x19 | 0x29 | 0x39 => {
                let index = ((opcode >> 4) & 0x3) as usize;

                let r16 = R16_GROUP_ONE[index];
                self.add_hl_r16(r16);
            }

            // LD (r16), A
            0x02 | 0x12 | 0x22 | 0x32 => {
                let index = ((opcode >> 4) & 0x3) as usize;

                let r16 = R16_GROUP_TWO[index];
                let addr = self.r.read_r16(r16);

                bus.write_byte(addr, self.r.a);
            }

            // LD A, (r16)
            0x0A | 0x1A | 0x2A | 0x3A => {
                let index = ((opcode >> 4) & 0x3) as usize;

                let r16 = R16_GROUP_TWO[index];
                let addr = self.r.read_r16(r16);

                self.r.a = bus.read_byte(addr);
            }

            // INC r16
            0x03 | 0x13 | 0x23 | 0x33 => {
                let reg = R16_GROUP_ONE[((opcode >> 4) & 0x3) as usize];
                let value = self.r.read_r16(reg);

                self.r.write_r16(reg, value.wrapping_add(1));
            }

            // DEC r16
            0x0B | 0x1B | 0x2B | 0x3B => {
                let reg = R16_GROUP_ONE[((opcode >> 4) & 0x3) as usize];
                let value = self.r.read_r16(reg);

                self.r.write_r16(reg, value.wrapping_sub(1));
            }

            // INC r8
            0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => {
                let bit_rep = (opcode >> 3) & 0x7;

                let r8 = unsafe { std::mem::transmute(bit_rep) };
                self.inc_r8(bus, r8);
            }

            // DEC r8
            0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => {
                let bit_rep = (opcode >> 3) & 0x7;

                let r8 = unsafe { std::mem::transmute(bit_rep) };
                self.dec_r8(bus, r8);
            }

            // LD r8, u8
            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                let bit_rep = (opcode >> 3) & 0x7;

                let r8 = unsafe { std::mem::transmute(bit_rep) };
                let imm = self.imm_byte(bus);

                self.r.write_r8(r8, bus, imm);
            }

            // MISC OPS
            0x07 | 0x17 | 0x27 | 0x37 | 0x0F | 0x1F | 0x2F | 0x3F => {
                let index = ((opcode >> 3) & 0x7) as usize;

                OPCODE_GROUP_ONE[index](self);
            }

            // HALT
            0x76 => self.halted = true,

            // LD r8, r8
            0x40..=0x7F if opcode != 0x76 => {
                let src_bit_rep = opcode & 0x7;
                let dest_bit_rep = (opcode >> 3) & 0x7;

                let src = unsafe { std::mem::transmute(src_bit_rep) };
                let dest = unsafe { std::mem::transmute(dest_bit_rep) };

                let r8v = self.r.read_r8(src, bus);
                self.r.write_r8(dest, bus, r8v);
            }

            // ALU A, r8
            0x80..=0xBF => {
                let bit_rep = opcode & 0x7;
                let index = ((opcode >> 3) & 0x7) as usize;

                let r8 = unsafe { std::mem::transmute(bit_rep) };

                let r8v = self.r.read_r8(r8, bus);
                OPCODE_GROUP_TWO[index](self, r8v);
            }

            // RET (conditional)
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                self.conditional_ret(bus, (opcode >> 3) & 0x3);
            }

            // LD [FF00 + u8], A
            0xE0 => {
                let addr = (0xFF00u16).wrapping_add(self.imm_byte(bus) as u16);
                bus.write_byte(addr, self.r.a);
            }

            // ADD SP, i8
            0xE8 => {
                let offset = self.imm_byte(bus) as i8 as i16 as u16;
                let sp = self.r.sp;

                self.r.sp = sp.wrapping_add(offset);

                self.r.set_zf(false);
                self.r.set_nf(false);
                self.r.set_hf((offset & 0xF) + (sp & 0xF) > 0xF);
                self.r.set_cf((offset & 0xFF) + (sp & 0xFF) > 0xFF);
            }

            // LD A, [FF00 + u8]
            0xF0 => {
                let addr = (0xFF00u16).wrapping_add(self.imm_byte(bus) as u16);
                self.r.a = bus.read_byte(addr);
            }

            // LD HL, SP + i8
            0xF8 => {
                let offset = self.imm_byte(bus) as i8 as i16 as u16;
                let sp = self.r.sp;

                self.r.write_r16(Reg16::HL, sp.wrapping_add(offset));

                self.r.set_zf(false);
                self.r.set_nf(false);
                self.r.set_hf((offset & 0xF) + (sp & 0xF) > 0xF);
                self.r.set_cf((offset & 0xFF) + (sp & 0xFF) > 0xFF);
            }

            // POP r16
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let index = ((opcode >> 4) & 0x3) as usize;

                let r16 = R16_GROUP_THR[index];
                let popped = self.pop(bus);

                self.r.write_r16(r16, popped);
            }

            // RET (unconditional)
            0xC9 => self.ret(bus),

            // RETI
            0xD9 => {
                self.ret(bus);
                self.ime = true;
            }

            // JP HL
            0xE9 => {
                let addr = self.r.read_r16(Reg16::HL);
                self.jp(addr);
            }

            // LD SP, HL
            0xF9 => self.r.sp = self.r.read_r16(Reg16::HL),

            // JP (conditional)
            0xC2 | 0xD2 | 0xCA | 0xDA => {
                let condition = (opcode >> 3) & 0x3;
                let addr = self.imm_word(bus);

                self.conditional_jp(addr, condition);
            }

            0xE2 => {
                let addr = (0xFF00u16).wrapping_add(self.r.c as u16);
                bus.write_byte(addr, self.r.a);
            }

            // LD [u16], A
            0xEA => {
                let addr = self.imm_word(bus);
                bus.write_byte(addr, self.r.a);
            }

            0xF2 => {
                let addr = (0xFF00u16).wrapping_add(self.r.c as u16);
                self.r.a = bus.read_byte(addr);
            }

            // LD A, [u16]
            0xFA => {
                let addr = self.imm_word(bus);
                self.r.a = bus.read_byte(addr);
            }

            // JP u16
            0xC3 => {
                let addr = self.imm_word(bus);
                self.jp(addr);
            }

            // CB prefixed.
            0xCB => {
                let opcode = self.imm_byte(bus);

                match opcode {
                    0x00..=0x3F => {
                        let bit_rep = opcode & 0x7;
                        let index = ((opcode >> 3) & 0x7) as usize;

                        let r8 = unsafe { std::mem::transmute(bit_rep) };

                        OPCODE_GROUP_THR[index](self, bus, r8);
                    }

                    0x40..=0x7F => {
                        let bit_rep = opcode & 0x7;
                        let bit = (opcode >> 3) & 0x7;

                        let r8 = unsafe { std::mem::transmute(bit_rep) };

                        self.bit_r8(bus, r8, bit);
                    }

                    0x80..=0xBF => {
                        let bit_rep = opcode & 0x7;
                        let bit = (opcode >> 3) & 0x7;

                        let r8 = unsafe { std::mem::transmute(bit_rep) };

                        self.res_r8(bus, r8, bit);
                    }

                    0xC0..=0xFF => {
                        let bit_rep = opcode & 0x7;
                        let bit = (opcode >> 3) & 0x7;

                        let r8 = unsafe { std::mem::transmute(bit_rep) };

                        self.set_r8(bus, r8, bit);
                    }
                }
            }

            // DI
            0xF3 => self.ime = false,

            // EI
            0xFB => self.ime = true,

            // CALL (condition)
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                let condition = (opcode >> 3) & 0x3;
                self.conditional_call(bus, condition);
            }

            // PUSH r16
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let index = ((opcode >> 4) & 0x3) as usize;

                let r16 = R16_GROUP_THR[index];
                let push = self.r.read_r16(r16);

                self.push(bus, push);
            }

            // CALL u16
            0xCD => self.call(bus),

            // ALU A, u8
            0xC6 | 0xD6 | 0xE6 | 0xF6 | 0xCE | 0xDE | 0xEE | 0xFE => {
                let index = ((opcode >> 3) & 0x7) as usize;
                let value = self.imm_byte(bus);

                OPCODE_GROUP_TWO[index](self, value);
            }

            // RSTs
            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                // Construct jump vector.
                let vec = (opcode & 0b0011_1000) as u16;

                // Push current PC onto the stack.
                self.push(bus, self.r.pc);

                // Jump to the vector.
                self.r.pc = vec;
            }

            // Invalid opcodes.
            _ => eprintln!("Illegal operation code {:#04X} encountered.", opcode),
        }
    }
}
