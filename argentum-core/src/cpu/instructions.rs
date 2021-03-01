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
            0 => !self.r.get_flag(Flags::Z),
            1 => self.r.get_flag(Flags::Z),
            2 => !self.r.get_flag(Flags::C),
            3 => self.r.get_flag(Flags::C),

            _ => unreachable!(),
        }
    }

    /// NOP.
    pub fn nop(&self) {}

    /// LD (u16), SP.
    pub fn ld_u16_sp(&mut self, bus: &mut Bus) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        bus.write_byte(self.r.sp, lower);
        bus.write_byte(self.r.sp + 1, upper);
    }

    /// STOP.
    pub fn stop(&mut self) {
        self.r.pc += 1;
    }

    /// JR (unconditional).
    pub fn unconditional_jr(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16;

        self.r.pc = (self.r.pc as i16 + offset) as u16;
        self.internal_cycle(bus);
    }

    /// JR (conditional).
    pub fn conditional_jr(&mut self, bus: &mut Bus, condition: u8) {
        let offset = self.imm_byte(bus) as i8 as i16;

        if self.get_condition(condition) {
            self.r.pc = (self.r.pc as i16 + offset) as u16;
            self.internal_cycle(bus);
        }
    }
}
