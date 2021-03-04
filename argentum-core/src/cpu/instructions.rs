use super::{registers::Flags, CPU};
use crate::bus::Bus;

impl CPU {
    /// Match condition according to,
    /// 0 => NZ
    /// 1 => Z
    /// 2 => NC
    /// 3 => C
    fn get_condition(&self, condition: u8) -> bool {
        match condition {
            0 => !self.reg.get_flag(Flags::Z),
            1 => self.reg.get_flag(Flags::Z),
            2 => !self.reg.get_flag(Flags::C),
            3 => self.reg.get_flag(Flags::C),

            _ => unreachable!(),
        }
    }

    /// NOP.
    pub fn nop(&self) {}

    /// LD (u16), SP.
    pub fn ld_u16_sp(&mut self, bus: &mut Bus) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        let address = u16::from_le_bytes([lower, upper]);

        let [sp_lower, sp_upper] = self.reg.sp.to_le_bytes();

        bus.write_byte(address, sp_lower);
        bus.write_byte(address + 1, sp_upper);
    }

    /// STOP.
    pub fn stop(&mut self) {
        self.reg.pc += 1;
    }

    /// JR (unconditional).
    pub fn unconditional_jr(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16;

        self.reg.pc = (self.reg.pc as i16 + offset) as u16;
        self.internal_cycle(bus);
    }

    /// JR (conditional).
    pub fn conditional_jr(&mut self, bus: &mut Bus, condition: u8) {
        let offset = self.imm_byte(bus) as i8 as i16;

        if self.get_condition(condition) {
            self.reg.pc = (self.reg.pc as i16 + offset) as u16;
            self.internal_cycle(bus);
        }
    }

    /// LD R16, u16.
    pub fn ld_r16_u16(&mut self, bus: &mut Bus, r16: u8) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        self.write_r16::<1>(r16, u16::from_le_bytes([lower, upper]));
    }

    /// ADD HL, R16.
    pub fn add_hl_r16(&mut self, bus: &mut Bus, r16: u8) {
        let hl = self.reg.get_hl();
        let value = self.read_r16::<1>(r16);

        self.reg.set_hl(hl.wrapping_add(value));
        self.internal_cycle(bus);

        self.reg.set_flag(Flags::N, false);
        self.reg
            .set_flag(Flags::H, (hl & 0xFFF) + (value & 0xFFF) > 0xFFF);
        self.reg
            .set_flag(Flags::C, (hl as u32) + (value as u32) > 0xFFFF);
    }

    /// LD (R16), A.
    pub fn ld_r16_a(&mut self, bus: &mut Bus, r16: u8) {
        let addr = self.read_r16::<2>(r16);

        bus.write_byte(addr, self.reg.a);
    }

    /// LD A, (R16).
    pub fn ld_a_r16(&mut self, bus: &mut Bus, r16: u8) {
        let addr = self.read_r16::<2>(r16);

        self.reg.a = bus.read_byte(addr);
    }

    /// INC R16.
    pub fn inc_r16(&mut self, bus: &mut Bus, r16: u8) {
        let value = self.read_r16::<1>(r16);

        self.write_r16::<1>(r16, value.wrapping_add(1));
        self.internal_cycle(bus);
    }

    /// DEC R16.
    pub fn dec_r16(&mut self, bus: &mut Bus, r16: u8) {
        let value = self.read_r16::<1>(r16);

        self.write_r16::<1>(r16, value.wrapping_sub(1));
        self.internal_cycle(bus);
    }

    /// INC R8.
    pub fn inc_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.wrapping_add(1);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, (value & 0xF) + 0x1 > 0xF);
    }

    /// DEC R8.
    pub fn dec_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.wrapping_sub(1);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, true);
        self.reg.set_flag(Flags::H, value.trailing_zeros() >= 4);
    }

    /// LD R8, u8.
    pub fn ld_r8_u8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.imm_byte(bus);

        self.write_r8(bus, r8, value);
    }

    /// RLCA.
    pub fn rlca(&mut self) {
        let value = self.reg.a;
        let result = value.rotate_left(1);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, false);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x80) != 0);
    }

    /// RRCA.
    pub fn rrca(&mut self) {
        let value = self.reg.a;
        let result = value.rotate_right(1);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, false);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x01) != 0);
    }

    /// RLA.
    pub fn rla(&mut self) {
        let value = self.reg.a;
        let carry = self.reg.get_flag(Flags::C) as u8;
        let result = (value << 1) | carry;

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, false);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x80) != 0);
    }

    /// RRA.
    pub fn rra(&mut self) {
        let value = self.reg.a;
        let carry = self.reg.get_flag(Flags::C) as u8;
        let result = (value >> 1) | (carry << 7);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, false);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x01) != 0);
    }

    /// DA A.
    pub fn daa(&mut self) {
        let mut a = self.reg.a;

        if self.reg.get_flag(Flags::N) {
            if self.reg.get_flag(Flags::C) {
                a = a.wrapping_add(0xA0);
                self.reg.set_flag(Flags::C, true);
            }

            if self.reg.get_flag(Flags::H) {
                a = a.wrapping_add(0xFA);
            }
        } else {
            if self.reg.get_flag(Flags::C) || (a > 0x99) {
                a = a.wrapping_add(0x60);
                self.reg.set_flag(Flags::C, true);
            }

            if self.reg.get_flag(Flags::H) || ((a & 0xF) > 0x9) {
                a = a.wrapping_add(0x06);
            }
        }

        self.reg.a = a;
        self.reg.set_flag(Flags::Z, a == 0);
        self.reg.set_flag(Flags::H, false);
    }

    /// CPL.
    pub fn cpl(&mut self) {
        self.reg.a = !self.reg.a;

        self.reg.set_flag(Flags::N, true);
        self.reg.set_flag(Flags::H, true);
    }

    /// SCF.
    pub fn scf(&mut self) {
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, true);
    }

    /// CCF.
    pub fn ccf(&mut self) {
        let carry = self.reg.get_flag(Flags::C);

        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, !carry);
    }

    /// LD R8, R8.
    pub fn ld_r8_r8(&mut self, bus: &mut Bus, src: u8, dst: u8) {
        let value = self.read_r8(bus, src);

        self.write_r8(bus, dst, value);
    }

    /// ADD A, R8.
    pub fn add_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let result = a.wrapping_add(value);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, (a & 0xF) + (value & 0xF) > 0xF);
        self.reg
            .set_flag(Flags::C, (a as u16 + value as u16) > 0xFF);
    }

    /// ADC A, R8.
    pub fn adc_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let f = self.reg.get_flag(Flags::C) as u8;
        let result = a.wrapping_add(value).wrapping_add(f);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg
            .set_flag(Flags::H, (a & 0xF) + (value & 0xF) + f > 0xF);
        self.reg
            .set_flag(Flags::C, (a as u16) + (value as u16) + (f as u16) > 0xFF);
    }

    /// SUB A, R8.
    pub fn sub_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let result = self.reg.a.wrapping_sub(value);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, true);
        self.reg.set_flag(Flags::H, (a & 0xF) < (value & 0xF));
        self.reg.set_flag(Flags::C, (a as u16) < (value as u16));
    }

    /// SBC A, R8.
    pub fn sbc_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let f = self.reg.get_flag(Flags::C) as u8;
        let result = a.wrapping_sub(value).wrapping_sub(f);

        self.reg.a = result;

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, true);
        self.reg
            .set_flag(Flags::H, (a & 0xF) < ((value & 0xF) + (f & 0xF)));
        self.reg
            .set_flag(Flags::C, (a as u16) < (value as u16 + f as u16));
    }

    /// AND A, R8.
    pub fn and_r8(&mut self, value: u8) {
        self.reg.a &= value;

        self.reg.set_flag(Flags::Z, self.reg.a == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, true);
        self.reg.set_flag(Flags::C, false);
    }

    /// XOR A, R8.
    pub fn xor_r8(&mut self, value: u8) {
        self.reg.a ^= value;

        self.reg.set_flag(Flags::Z, self.reg.a == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, false);
    }

    /// OR A, R8.
    pub fn or_r8(&mut self, value: u8) {
        self.reg.a |= value;

        self.reg.set_flag(Flags::Z, self.reg.a == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, false);
    }

    /// CP A, R8.
    pub fn cp_r8(&mut self, value: u8) {
        let result = self.reg.a.wrapping_sub(value);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, true);
        self.reg
            .set_flag(Flags::H, (self.reg.a & 0xF) < (value & 0xF));
        self.reg
            .set_flag(Flags::C, (self.reg.a as u16) < (value as u16));
    }

    /// RET (unconditional).
    pub fn unconditional_ret(&mut self, bus: &mut Bus) {
        let lower = bus.read_byte(self.reg.sp);
        self.reg.sp += 1;

        let upper = bus.read_byte(self.reg.sp);
        self.reg.sp += 1;

        let return_address = u16::from_le_bytes([lower, upper]);

        self.internal_cycle(bus);
        self.reg.pc = return_address;
    }

    /// RET (conditional).
    pub fn conditional_ret(&mut self, bus: &mut Bus, condition: u8) {
        self.internal_cycle(bus);

        if self.get_condition(condition) {
            self.unconditional_ret(bus);
        }
    }

    /// LD [FF00 + u8], A.
    pub fn ld_io_u8_a(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as u16;

        bus.write_byte(0xFF00u16.wrapping_add(offset), self.reg.a);
    }

    /// ADD SP, i8.
    pub fn add_sp_i8(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16 as u16;
        let sp = self.reg.sp;

        self.reg.sp = sp.wrapping_add(offset);

        self.reg.set_flag(Flags::Z, false);
        self.reg.set_flag(Flags::N, false);
        self.reg
            .set_flag(Flags::H, (offset & 0xF) + (sp & 0xF) > 0xF);
        self.reg
            .set_flag(Flags::C, (offset & 0xFF) + (sp & 0xFF) > 0xFF);
    }

    /// LD A, [FF00 + u8].
    pub fn ld_a_io_u8(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as u16;

        self.reg.a = bus.read_byte(0xFF00u16.wrapping_add(offset));
    }

    /// LD HL, SP + i8.
    pub fn ld_hl_sp_i8(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16 as u16;
        let sp = self.reg.sp;

        self.reg.set_hl(sp.wrapping_add(offset));

        self.reg.set_flag(Flags::Z, false);
        self.reg.set_flag(Flags::N, false);
        self.reg
            .set_flag(Flags::H, (offset & 0xF) + (sp & 0xF) > 0xF);
        self.reg
            .set_flag(Flags::C, (offset & 0xFF) + (sp & 0xFF) > 0xFF);
    }

    /// POP R16.
    pub fn pop_r16(&mut self, bus: &mut Bus, r16: u8) {
        let lower = bus.read_byte(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let upper = bus.read_byte(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let value = u16::from_le_bytes([lower, upper]);

        self.write_r16::<3>(r16, value);
    }

    /// RETI.
    pub fn reti(&mut self, bus: &mut Bus) {
        self.unconditional_ret(bus);
        self.ime = true;
    }

    /// JP HL.
    pub fn jp_hl(&mut self) {
        self.reg.pc = self.reg.get_hl();
    }

    /// LD SP, HL.
    pub fn ld_sp_hl(&mut self) {
        self.reg.sp = self.reg.get_hl();
    }

    /// JP (unconditional).
    pub fn unconditional_jp(&mut self, bus: &mut Bus) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        let jump_address = u16::from_le_bytes([lower, upper]);

        self.internal_cycle(bus);
        self.reg.pc = jump_address;
    }

    /// JP (conditional).
    pub fn conditional_jp(&mut self, bus: &mut Bus, condition: u8) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        let jump_address = u16::from_le_bytes([lower, upper]);

        if self.get_condition(condition) {
            self.internal_cycle(bus);
            self.reg.pc = jump_address;
        }
    }

    /// RLC R8.
    pub fn rlc_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.rotate_left(1);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x80) != 0);
    }

    /// RRC R8.
    pub fn rrc_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.rotate_right(1);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x01) != 0);
    }

    /// RL R8.
    pub fn rl_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let carry = self.reg.get_flag(Flags::C) as u8;
        let result = (value << 1) | carry;

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x80) != 0);
    }

    /// RR r8.
    pub fn rr_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let carry = self.reg.get_flag(Flags::C) as u8;
        let result = (value >> 1) | (carry << 7);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x01) != 0);
    }

    /// SLA R8.
    pub fn sla_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value << 1;

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x80) != 0);
    }

    /// SRA R8.
    pub fn sra_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = (value >> 1) | (value & 0x80);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x01) != 0);
    }

    /// SWAP R8.
    pub fn swap_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = (value << 4) | (value >> 4);

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, false);
    }

    // SRL R8.
    pub fn srl_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value >> 1;

        self.write_r8(bus, r8, result);

        self.reg.set_flag(Flags::Z, result == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, false);
        self.reg.set_flag(Flags::C, (value & 0x01) != 0);
    }

    // BIT bit, R8.
    pub fn bit_r8(&mut self, bus: &mut Bus, r8: u8, bit: u8) {
        let value = self.read_r8(bus, r8) & (1 << bit);

        self.reg.set_flag(Flags::Z, value == 0);
        self.reg.set_flag(Flags::N, false);
        self.reg.set_flag(Flags::H, true);
    }

    // RES bit, R8.
    pub fn res_r8(&mut self, bus: &mut Bus, r8: u8, bit: u8) {
        let value = self.read_r8(bus, r8);
        let mask = !(1 << bit);
        let result = value & mask;

        self.write_r8(bus, r8, result);
    }

    /// SET bit, R8.
    pub fn set_r8(&mut self, bus: &mut Bus, r8: u8, bit: u8) {
        let value = self.read_r8(bus, r8) | (1 << bit);

        self.write_r8(bus, r8, value);
    }

    /// CALL (conditional).
    pub fn conditional_call(&mut self, bus: &mut Bus, condition: u8) {
        let lower_imm = self.imm_byte(bus);
        let upper_imm = self.imm_byte(bus);

        let address = u16::from_le_bytes([lower_imm, upper_imm]);

        if self.get_condition(condition) {
            self.internal_cycle(bus);

            let [lower, upper] = self.reg.pc.to_le_bytes();

            self.reg.sp = self.reg.sp.wrapping_sub(1);
            bus.write_byte(self.reg.sp, upper);

            self.reg.sp = self.reg.sp.wrapping_sub(1);
            bus.write_byte(self.reg.sp, lower);

            self.reg.pc = address;
        }
    }

    /// CALL (unconditional).
    pub fn unconditional_call(&mut self, bus: &mut Bus) {
        let lower_imm = self.imm_byte(bus);
        let upper_imm = self.imm_byte(bus);

        let address = u16::from_le_bytes([lower_imm, upper_imm]);

        self.internal_cycle(bus);

        let [lower, upper] = self.reg.pc.to_le_bytes();

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        bus.write_byte(self.reg.sp, upper);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        bus.write_byte(self.reg.sp, lower);

        self.reg.pc = address;
    }

    /// PUSH R16.
    pub fn push_r16(&mut self, bus: &mut Bus, r16: u8) {
        let value = self.read_r16::<3>(r16);
        let [lower, upper] = value.to_le_bytes();

        self.internal_cycle(bus);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        bus.write_byte(self.reg.sp, upper);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        bus.write_byte(self.reg.sp, lower);
    }
}
