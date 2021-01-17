//! Contains implementations of opcodes.

use super::*;

impl Cpu {
    /// Match condition according to,
    /// 0 => NZ
    /// 1 => Z
    /// 2 => NC
    /// 3 => C
    fn get_condition(&self, condition: u8) -> bool {
        match condition {
            0 => !self.r.get_zf(),
            1 => self.r.get_zf(),
            2 => !self.r.get_cf(),
            3 => self.r.get_cf(),

            _ => unreachable!(),
        }
    }

    /// Do nothing. No OPeration.
    pub fn nop(&self) {}

    /// LD (u16), SP
    pub fn ld_u16_sp(&mut self, bus: &mut Bus) {
        // Read u16 immediate value.
        let addr = self.imm_word(bus);

        // Write to the address u16, the value of SP.
        bus.write_word(addr, self.r.sp);
    }

    /// STOP
    /// Its length is 2 bytes for some reason.
    pub fn stop(&mut self, bus: &mut Bus) {
        // Read 1 byte, as STOP is 2 bytes long.
        self.imm_byte(bus);
    }

    /// Push a value onto the stack.
    pub fn push(&mut self, bus: &mut Bus, value: u16) {
        // Stack grows downwards, and each element
        // is one word long.
        self.r.sp -= 2;

        // Write the value to the stack.
        bus.write_word(self.r.sp, value);
    }

    /// Pop a value off the stack.
    pub fn pop(&mut self, bus: &mut Bus) -> u16 {
        // Pop the value off, by increasing SP by 2.
        // This will cause the previous value to be
        // overwrited.
        let value = bus.read_word(self.r.sp);
        self.r.sp += 2;

        value
    }

    /// Unconditional jump to the given address.
    pub fn jp(&mut self, addr: u16) {
        self.r.pc = addr;
    }

    /// Conditional jump to given address.
    pub fn conditional_jp(&mut self, addr: u16, condition: u8) {
        let is_satisfied = self.get_condition(condition);

        if is_satisfied {
            self.jp(addr);
        }
    }

    /// Unconditional return from a subroutine.
    pub fn ret(&mut self, bus: &mut Bus) {
        self.r.pc = self.pop(bus);
    }

    /// Conditional return from a subroutine.
    pub fn conditional_ret(&mut self, bus: &mut Bus, condition: u8) {
        let is_satisfied = self.get_condition(condition);

        if is_satisfied {
            self.ret(bus);
        }
    }

    /// Unconditional call to a subroutine.
    pub fn call(&mut self, bus: &mut Bus) {
        let addr = self.imm_word(bus);

        // Push the current PC to stack.
        self.push(bus, self.r.pc);

        // Jump to the address.
        self.r.pc = addr;
    }

    /// Conditional call to a subroutine.
    pub fn conditional_call(&mut self, bus: &mut Bus, condition: u8) {
        let addr = self.imm_word(bus);
        let is_satisfied = self.get_condition(condition);

        if is_satisfied {
            self.push(bus, self.r.pc);
            self.r.pc = addr;
        }
    }

    /// Unconditional relative jump.
    pub fn jr(&mut self, bus: &mut Bus) {
        // Convert the offset to a signed value.
        let offset = self.imm_byte(bus) as i8 as i16;

        // Add a signed value to PC.
        self.r.pc = (self.r.pc as i16 + offset) as u16;
    }

    /// Conditional relative jump.
    pub fn conditional_jr(&mut self, bus: &mut Bus, condition: u8) {
        let offset = self.imm_byte(bus) as i8 as i16;
        let is_satisfied = self.get_condition(condition);

        if is_satisfied {
            self.r.pc = (self.r.pc as i16 + offset) as u16;
        }
    }

    /// ADD HL, r16
    pub fn add_hl_r16(&mut self, r16: Reg16) {
        let hl = self.r.read_r16(Reg16::HL);
        let rr = self.r.read_r16(r16);

        self.r.write_r16(Reg16::HL, hl.wrapping_add(rr));

        self.r.set_nf(false);
        self.r.set_hf((hl & 0xFFF) + (rr & 0xFFF) > 0xFFF);
        self.r.set_cf((hl as u32) + (rr as u32) > 0xFFFF);
    }

    /// INC r8
    pub fn inc_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v.wrapping_add(1);

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf((r8v & 0xF) + 0x1 > 0xF);
    }

    /// DEC r8
    pub fn dec_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v.wrapping_sub(1);

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(true);
        self.r.set_hf(r8v.trailing_zeros() >= 4);
    }

    /// ADD A, r8
    pub fn add_r8(&mut self, r8v: u8) {
        let a = self.r.a;
        let (result, carry) = a.overflowing_add(r8v);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf((a & 0xF) + (r8v & 0xF) > 0xF);
        self.r.set_cf(carry);
    }

    /// ADC A, r8
    pub fn adc_r8(&mut self, r8v: u8) {
        let a = self.r.a;
        let f = self.r.get_cf() as u8;
        let result = a.wrapping_add(r8v).wrapping_add(f);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf((a & 0xF) + (r8v & 0xF) + f > 0xF);
        self.r.set_cf((a as u16) + (r8v as u16) + (f as u16) > 0xFF);
    }

    /// SUB A, r8
    pub fn sub_r8(&mut self, r8v: u8) {
        let a = self.r.a;
        let (result, carry) = a.overflowing_sub(r8v);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(true);
        self.r.set_hf(a.trailing_zeros() >= 4);
        self.r.set_cf(carry);
    }

    /// SBC A, r8
    pub fn sbc_r8(&mut self, r8v: u8) {
        let a = self.r.a;
        let f = self.r.get_cf() as u8;
        let result = a.wrapping_sub(r8v).wrapping_sub(f);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(true);
        self.r.set_hf(a.trailing_zeros() >= 4);
        self.r.set_cf((a as u16) < (r8v as u16 + f as u16));
    }

    /// AND A, r8
    pub fn and_r8(&mut self, r8v: u8) {
        self.r.a &= r8v;

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf(true);
        self.r.set_cf(false);
    }

    /// XOR A, r8
    pub fn xor_r8(&mut self, r8v: u8) {
        self.r.a ^= r8v;

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(false);
    }

    /// OR A, r8
    pub fn or_r8(&mut self, r8v: u8) {
        self.r.a |= r8v;

        self.r.set_zf(self.r.a == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(false);
    }

    /// CP A, r8
    pub fn cp_r8(&mut self, r8v: u8) {
        let a = self.r.a;
        let (result, carry) = a.overflowing_sub(r8v);

        self.r.set_zf(result == 0);
        self.r.set_nf(true);
        self.r.set_hf(a.trailing_zeros() >= 4);
        self.r.set_cf(carry);
    }

    // RLC r8
    pub fn rlc_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v.rotate_left(1);

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x80) != 0);
    }

    // RRC r8
    pub fn rrc_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v.rotate_right(1);

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x01) != 0);
    }

    // RL r8
    pub fn rl_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let carry = self.r.get_cf() as u8;
        let result = (r8v << 1) | carry;

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x80) != 0);
    }

    // RR r8
    pub fn rr_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let carry = self.r.get_cf() as u8;
        let result = (r8v >> 1) | (carry << 7);

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x01) != 0);
    }

    // SLA r8
    pub fn sla_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v << 1;

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x80) != 0);
    }

    // SRA r8
    pub fn sra_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = (r8v >> 1) | (r8v & 0x80);

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x01) != 0);
    }

    // SWAP r8
    pub fn swap_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v.reverse_bits();

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(false);
    }

    // SRL r8
    pub fn srl_r8(&mut self, bus: &mut Bus, r8: Reg8) {
        let r8v = self.r.read_r8(r8, bus);
        let result = r8v >> 1;

        self.r.write_r8(r8, bus, result);

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x01) != 0);
    }

    // BIT bit, r8
    pub fn bit_r8(&mut self, bus: &mut Bus, r8: Reg8, bit: u8) {
        let r8v = self.r.read_r8(r8, bus) & (1 << bit);

        self.r.set_zf(r8v == 0);
        self.r.set_nf(false);
        self.r.set_hf(true);
    }

    // RES bit, r8
    pub fn res_r8(&mut self, bus: &mut Bus, r8: Reg8, bit: u8) {
        let r8v = self.r.read_r8(r8, bus);
        let mask = !(1 << bit);
        let result = r8v & mask;

        self.r.write_r8(r8, bus, result);
    }

    // SET bit, r8
    pub fn set_r8(&mut self, bus: &mut Bus, r8: Reg8, bit: u8) {
        let r8v = self.r.read_r8(r8, bus) | (1 << bit);
        self.r.write_r8(r8, bus, r8v);
    }

    /// RLCA
    pub fn rlca(&mut self) {
        let r8v = self.r.a;
        let result = r8v.rotate_left(1);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x80) != 0);
    }

    /// RLCA
    pub fn rrca(&mut self) {
        let r8v = self.r.a;
        let result = r8v.rotate_right(1);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x01) != 0);
    }

    /// RLA
    pub fn rla(&mut self) {
        let r8v = self.r.a;
        let carry = self.r.get_cf() as u8;
        let result = (r8v << 1) | carry;

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x80) != 0);
    }

    /// RRA
    pub fn rra(&mut self) {
        let r8v = self.r.a;
        let carry = self.r.get_cf() as u8;
        let result = (r8v >> 1) | (carry << 7);

        self.r.a = result;

        self.r.set_zf(result == 0);
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf((r8v & 0x01) != 0);
    }

    /// TODO
    pub fn daa(&mut self) {}

    /// CPL
    pub fn cpl(&mut self) {
        self.r.a = !self.r.a;

        self.r.set_nf(true);
        self.r.set_hf(true);
    }

    /// SCF
    pub fn scf(&mut self) {
        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(true);
    }

    /// CCF
    pub fn ccf(&mut self) {
        let carry = self.r.get_cf();

        self.r.set_nf(false);
        self.r.set_hf(false);
        self.r.set_cf(!carry);
    }
}
