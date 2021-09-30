use crate::bus::Bus;
use crate::cpu::Cpu;

impl Cpu {
    /// Decode the provided CPU instruction and execute it.
    pub fn decode_and_execute(&mut self, bus: &mut Bus, instruction: u8) {
        match instruction {
            0x00 => self.nop(),

            0x08 => self.ld_u16_sp(bus),

            0x10 => self.stop(bus),

            0x18 => self.unconditional_jr(bus),

            0x20 | 0x28 | 0x30 | 0x38 => {
                let condition = (instruction >> 3) & 0x3;

                self.conditional_jr(bus, condition);
            }

            0x01 | 0x11 | 0x21 | 0x31 => {
                let r16 = (instruction >> 4) & 0x3;

                self.ld_r16_u16(bus, r16);
            }

            0x09 | 0x19 | 0x29 | 0x39 => {
                let r16 = (instruction >> 4) & 0x3;

                self.add_hl_r16(bus, r16);
            }

            0x02 | 0x12 | 0x22 | 0x32 => {
                let r16 = (instruction >> 4) & 0x3;

                self.ld_r16_a(bus, r16)
            }

            0x0A | 0x1A | 0x2A | 0x3A => {
                let r16 = (instruction >> 4) & 0x3;

                self.ld_a_r16(bus, r16);
            }

            0x03 | 0x13 | 0x23 | 0x33 => {
                let r16 = (instruction >> 4) & 0x3;

                self.inc_r16(bus, r16);
            }

            0x0B | 0x1B | 0x2B | 0x3B => {
                let r16 = (instruction >> 4) & 0x3;

                self.dec_r16(bus, r16);
            }

            0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => {
                let r8 = (instruction >> 3) & 0x7;

                self.inc_r8(bus, r8);
            }

            0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => {
                let r8 = (instruction >> 3) & 0x7;

                self.dec_r8(bus, r8);
            }

            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                let r8 = (instruction >> 3) & 0x7;

                self.ld_r8_u8(bus, r8);
            }

            0x07 | 0x17 | 0x27 | 0x37 | 0x0F | 0x1F | 0x2F | 0x3F => {
                let operation = (instruction >> 3) & 0x7;

                match operation {
                    0 => self.rlca(),
                    1 => self.rrca(),
                    2 => self.rla(),
                    3 => self.rra(),
                    4 => self.daa(),
                    5 => self.cpl(),
                    6 => self.scf(),
                    7 => self.ccf(),

                    _ => unreachable!(),
                }
            }

            0x76 => self.halt(),

            0x40..=0x7F if instruction != 0x76 => {
                let src = instruction & 0x7;
                let dst = (instruction >> 3) & 0x7;

                self.ld_r8_r8(bus, src, dst);
            }

            0x80..=0xBF => {
                let r8 = instruction & 0x7;
                let value = self.read_r8(bus, r8);

                let operation = (instruction >> 3) & 0x7;

                match operation {
                    0 => self.add_r8(value),
                    1 => self.adc_r8(value),
                    2 => self.sub_r8(value),
                    3 => self.sbc_r8(value),
                    4 => self.and_r8(value),
                    5 => self.xor_r8(value),
                    6 => self.or_r8(value),
                    7 => self.cp_r8(value),

                    _ => unreachable!(),
                }
            }

            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                let condition = (instruction >> 3) & 0x3;

                self.conditional_ret(bus, condition);
            }

            0xE0 => self.ld_io_u8_a(bus),

            0xE8 => self.add_sp_i8(bus),

            0xF0 => self.ld_a_io_u8(bus),

            0xF8 => self.ld_hl_sp_i8(bus),

            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let r16 = (instruction >> 4) & 0x3;

                self.pop_r16(bus, r16);
            }

            0xC9 => self.unconditional_ret(bus),

            0xD9 => self.reti(bus),

            0xE9 => self.jp_hl(),

            0xF9 => self.ld_sp_hl(bus),

            0xC2 | 0xD2 | 0xCA | 0xDA => {
                let condition = (instruction >> 3) & 0x3;

                self.conditional_jp(bus, condition);
            }

            0xE2 => self.ld_io_c_a(bus),

            0xEA => self.ld_u16_a(bus),

            0xF2 => self.ld_a_io_c(bus),

            0xFA => self.ld_a_u16(bus),

            0xC3 => self.unconditional_jp(bus),

            0xCB => {
                let opcode = self.imm_byte(bus);

                match opcode {
                    0x00..=0x3F => {
                        let r8 = opcode & 0x7;
                        let operation = (opcode >> 3) & 0x7;

                        match operation {
                            0 => self.rlc_r8(bus, r8),
                            1 => self.rrc_r8(bus, r8),
                            2 => self.rl_r8(bus, r8),
                            3 => self.rr_r8(bus, r8),
                            4 => self.sla_r8(bus, r8),
                            5 => self.sra_r8(bus, r8),
                            6 => self.swap_r8(bus, r8),
                            7 => self.srl_r8(bus, r8),

                            _ => unreachable!(),
                        }
                    }

                    0x40..=0x7F => {
                        let r8 = opcode & 0x7;
                        let bit = (opcode >> 3) & 0x7;

                        self.bit_r8(bus, r8, bit);
                    }

                    0x80..=0xBF => {
                        let r8 = opcode & 0x7;
                        let bit = (opcode >> 3) & 0x7;

                        self.res_r8(bus, r8, bit);
                    }

                    0xC0..=0xFF => {
                        let r8 = opcode & 0x7;
                        let bit = (opcode >> 3) & 0x7;

                        self.set_r8(bus, r8, bit);
                    }
                }
            }

            0xF3 => self.di(),

            0xFB => self.ei(),

            0xC4 | 0xCC | 0xD4 | 0xDC => {
                let condition = (instruction >> 3) & 0x3;

                self.conditional_call(bus, condition);
            }

            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let r16 = (instruction >> 4) & 0x3;

                self.push_r16(bus, r16)
            }

            0xCD => self.unconditional_call(bus),

            0xC6 | 0xD6 | 0xE6 | 0xF6 | 0xCE | 0xDE | 0xEE | 0xFE => {
                let operation = (instruction >> 3) & 0x7;
                let value = self.imm_byte(bus);

                match operation {
                    0 => self.add_r8(value),
                    1 => self.adc_r8(value),
                    2 => self.sub_r8(value),
                    3 => self.sbc_r8(value),
                    4 => self.and_r8(value),
                    5 => self.xor_r8(value),
                    6 => self.or_r8(value),
                    7 => self.cp_r8(value),

                    _ => unreachable!(),
                }
            }

            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                let vec = (instruction & 0b0011_1000) as u16;

                self.rst(bus, vec);
            }

            _ => {}
        }
    }
}
