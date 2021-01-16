//! Contains implementations of opcodes.

use super::*;

impl Cpu {
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

    /// Unconditional relative jump.
    pub fn jr(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16;
        self.pc = (self.pc as i16 + offset) as u16;
    }

    /// Conditional relative jump.
    pub fn conditional_jr(&mut self, bus: &mut Bus, condition: u8) {
        let is_satisfied = match condition {
            0 => !self.r.get_zf(),
            1 => self.r.get_zf(),
            2 => !self.r.get_cf(),
            3 => self.r.get_cf(),

            _ => unreachable!(),
        };

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
        self.r.set_hf(value.trailing_zeros() >= 4);
    }
}
