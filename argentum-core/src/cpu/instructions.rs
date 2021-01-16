//! Contains implementations of opcodes.

use super::*;

impl Cpu {
    fn get_condition(&self, condition: u8) -> bool {
        match condition {
            0 => !self.r.get_zf(),
            1 => self.r.get_zf(),
            2 => !self.r.get_cf(),
            3 => self.r.get_cf(),

            _ => unreachable!(),
        }
    }

    /// Does nothing, NO oPeration.
    pub fn nop(&self) {}

    /// LD (u16), SP
    pub fn ld_u16_sp(&mut self, bus: &mut Bus) {
        let addr = self.imm_word(bus);
        bus.write_word(addr, self.r.sp);
    }

    /// STOP, (is of 2 bytes for some reason)
    pub fn stop(&mut self, bus: &mut Bus) {
        self.imm_byte(bus);
    }

    /// Push a value onto the stack.
    pub fn push(&mut self, bus: &mut Bus, value: u16) {
        self.r.sp -= 2;
        bus.write_word(self.r.sp, value);
    }

    /// Pop a value off the stack.
    pub fn pop(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.r.sp);
        self.r.sp += 2;

        value
    }

    /// Unconditional jump to given address.
    pub fn jp(&mut self, addr: u16) {
        self.pc = addr;
    }

    /// Conditional jump to given address.
    pub fn conditional_jp(&mut self, addr: u16, condition: u8) {
        let is_satisfied = self.get_condition(condition);

        if is_satisfied {
            self.jp(addr);
        }
    }

    /// Unconditional return from a routine.
    pub fn ret(&mut self, bus: &mut Bus) {
        self.pc = self.pop(bus);
    }

    /// Conditional return from a routine.
    pub fn conditional_ret(&mut self, bus: &mut Bus, condition: u8) {
        let is_satisfied = self.get_condition(condition);

        if is_satisfied {
            self.ret(bus);
        }
    }

    /// Unconditional relative jump.
    pub fn jr(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16;
        self.pc = (self.pc as i16 + offset) as u16;
    }

    /// Conditional relative jump.
    pub fn conditional_jr(&mut self, bus: &mut Bus, condition: u8) {
        let is_satisfied = self.get_condition(condition);

        let offset = self.imm_byte(bus) as i8 as i16;

        if is_satisfied {
            self.pc = (self.pc as i16 + offset) as u16;
        }
    }

    /// ADD HL, r16
    pub fn add_hl_rr(&mut self, rr: Reg16) {
        let hl = self.r.read_rr(Reg16::HL);
        let rr = self.r.read_rr(rr);

        self.r.write_rr(Reg16::HL, hl.wrapping_add(rr));

        self.r.set_nf(false);
        self.r
            .set_hf(((hl & 0xFFF) + (rr & 0xFFF)) & 0x1000 >= 0x1000);
        self.r.set_cf((hl as u32 + rr as u32) > 0xFFFF);
    }

    /// INC r8
    pub fn inc_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let value = self.r.get_r8(r8, bus);
        let res = value.wrapping_add(1);

        self.r.set_r8(r8, bus, res);

        self.r.set_zf(res == 0);
        self.r.set_nf(false);
        self.r.set_hf((value & 0xF) + 0x1 > 0xF);
    }

    /// DEC r8
    pub fn dec_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let value = self.r.get_r8(r8, bus);
        let res = value.wrapping_sub(1);

        self.r.set_r8(r8, bus, res);

        self.r.set_zf(res == 0);
        self.r.set_nf(false);
        self.r.set_hf((value & 0xF) < 1);
    }

    /// ADD A, r8
    pub fn add_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let a = self.r.a;
        let value = self.r.get_r8(r8, bus);

        self.r.a = a.wrapping_add(value);

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf((value & 0xF) + (a & 0xF) > 0xF);
        self.r.set_cf(value as u16 + a as u16 > 0xFF);
    }

    /// ADC A, r8
    pub fn adc_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let a = self.r.a;
        let value = self.r.get_r8(r8, bus);
        let flag = if self.r.get_cf() { 1 } else { 0 };

        self.r.a = a.wrapping_add(value).wrapping_add(flag);

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf((value & 0xF) + (a & 0xF) + flag > 0xF);
        self.r.set_cf(value as u16 + a as u16 + flag as u16 > 0xFF);
    }

    /// SUB A, r8
    pub fn sub_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let a = self.r.a;
        let value = self.r.get_r8(r8, bus);

        self.r.a = a.wrapping_sub(value);

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(true);
        self.r.set_hf((a & 0xF) < (value & 0xF));
        self.r.set_cf((a as u16) < (value as u16));
    }

    /// SBC A, r8
    pub fn sbc_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let a = self.r.a;
        let flag = if self.r.get_cf() { 1 } else { 0 };
        let value = self.r.get_r8(r8, bus);

        self.r.a = a.wrapping_sub(value).wrapping_sub(flag);

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(true);
        self.r.set_hf((a & 0xF) < ((value & 0xF) + flag));
        self.r.set_cf((a as u16) < (value as u16 + flag as u16));
    }

    /// AND A, r8
    pub fn and_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let value = self.r.get_r8(r8, bus);

        self.r.a &= value;

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf(true);
        self.r.set_cf(false);
    }

    /// XOR A, r8
    pub fn xor_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let value = self.r.get_r8(r8, bus);

        self.r.a ^= value;

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(false);
    }

    /// OR A, r8
    pub fn or_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let value = self.r.get_r8(r8, bus);

        self.r.a |= value;

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(false);
    }

    /// CP A, r8
    pub fn cp_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let value = self.r.get_r8(r8, bus);
        let result = self.r.a.wrapping_sub(value);

        self.r.set_zf(result == 0);
        self.r.set_nf(true);
        self.r.set_hf((self.r.a & 0xF) < (value & 0xF));
        self.r.set_cf((self.r.a as u16) < (value as u16));
    }
}
