use crate::bus::Bus;
use crate::cpu::Cpu;

impl Cpu {
    pub fn nop(&self) {}

    pub fn ld_u16_sp(&mut self, bus: &mut Bus) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        let address = u16::from_le_bytes([lower, upper]);

        let [sp_lower, sp_upper] = self.reg.sp.to_le_bytes();

        self.write_byte(bus, address, sp_lower);
        self.write_byte(bus, address + 1, sp_upper);
    }

    pub fn stop(&mut self, bus: &mut Bus) {
        if self.is_cgb && bus.is_preparing_switch() {
            bus.perform_speed_switch();
        }

        self.reg.pc = self.reg.pc.wrapping_add(1);
    }

    pub fn unconditional_jr(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16;

        self.reg.pc = (self.reg.pc as i16 + offset) as u16;
        self.internal_cycle(bus);
    }

    pub fn conditional_jr(&mut self, bus: &mut Bus, condition: u8) {
        let offset = self.imm_byte(bus) as i8 as i16;

        if self.reg.check_condition(condition) {
            self.reg.pc = (self.reg.pc as i16 + offset) as u16;
            self.internal_cycle(bus);
        }
    }

    pub fn ld_r16_u16(&mut self, bus: &mut Bus, r16: u8) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        self.write_r16::<1>(r16, u16::from_le_bytes([lower, upper]));
    }

    pub fn add_hl_r16(&mut self, bus: &mut Bus, r16: u8) {
        let hl = self.reg.get_hl();
        let value = self.read_r16::<1>(r16);

        self.reg.set_hl(hl.wrapping_add(value));
        self.internal_cycle(bus);

        self.reg.nf = false;
        self.reg.hf = ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF;
        self.reg.cf = ((hl as u32) + (value as u32)) > 0xFFFF;
    }

    pub fn ld_r16_a(&mut self, bus: &mut Bus, r16: u8) {
        let addr = self.read_r16::<2>(r16);

        self.write_byte(bus, addr, self.reg.a);
    }

    pub fn ld_a_r16(&mut self, bus: &mut Bus, r16: u8) {
        let addr = self.read_r16::<2>(r16);

        self.reg.a = self.read_byte(bus, addr);
    }

    pub fn inc_r16(&mut self, bus: &mut Bus, r16: u8) {
        let value = self.read_r16::<1>(r16);

        self.write_r16::<1>(r16, value.wrapping_add(1));
        self.internal_cycle(bus);
    }

    pub fn dec_r16(&mut self, bus: &mut Bus, r16: u8) {
        let value = self.read_r16::<1>(r16);

        self.write_r16::<1>(r16, value.wrapping_sub(1));
        self.internal_cycle(bus);
    }

    pub fn inc_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.wrapping_add(1);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = ((value & 0xF) + 0x1) > 0xF;
    }

    pub fn dec_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.wrapping_sub(1);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = true;
        self.reg.hf = value.trailing_zeros() >= 4;
    }

    pub fn ld_r8_u8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.imm_byte(bus);

        self.write_r8(bus, r8, value);
    }

    pub fn rlca(&mut self) {
        let value = self.reg.a;
        let result = value.rotate_left(1);

        self.reg.a = result;

        self.reg.zf = false;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x80) != 0;
    }

    pub fn rrca(&mut self) {
        let value = self.reg.a;
        let result = value.rotate_right(1);

        self.reg.a = result;

        self.reg.zf = false;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x01) != 0;
    }

    pub fn rla(&mut self) {
        let value = self.reg.a;
        let carry = self.reg.cf as u8;
        let result = (value << 1) | carry;

        self.reg.a = result;

        self.reg.zf = false;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x80) != 0;
    }

    pub fn rra(&mut self) {
        let value = self.reg.a;
        let carry = self.reg.cf as u8;
        let result = (value >> 1) | (carry << 7);

        self.reg.a = result;

        self.reg.zf = false;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x01) != 0;
    }

    pub fn daa(&mut self) {
        let mut a = self.reg.a;

        if self.reg.nf {
            if self.reg.cf {
                a = a.wrapping_add(0xA0);
                self.reg.cf = true;
            }

            if self.reg.hf {
                a = a.wrapping_add(0xFA);
            }
        } else {
            if self.reg.cf || (a > 0x99) {
                a = a.wrapping_add(0x60);
                self.reg.cf = true;
            }

            if self.reg.hf || ((a & 0xF) > 0x9) {
                a = a.wrapping_add(0x06);
            }
        }

        self.reg.a = a;

        self.reg.zf = a == 0;
        self.reg.hf = false;
    }

    pub fn cpl(&mut self) {
        self.reg.a = !self.reg.a;

        self.reg.nf = true;
        self.reg.hf = true;
    }

    pub fn scf(&mut self) {
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = true;
    }

    pub fn ccf(&mut self) {
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = !self.reg.cf;
    }

    pub fn ld_r8_r8(&mut self, bus: &mut Bus, src: u8, dst: u8) {
        let value = self.read_r8(bus, src);

        self.write_r8(bus, dst, value);
    }

    pub fn add_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let result = a.wrapping_add(value);

        self.reg.a = result;

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = ((a & 0xF) + (value & 0xF)) > 0xF;
        self.reg.cf = ((a as u16) + (value as u16)) > 0xFF;
    }

    pub fn adc_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let carry = self.reg.cf as u8;
        let result = a.wrapping_add(value).wrapping_add(carry);

        self.reg.a = result;

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = ((a & 0xF) + (value & 0xF) + carry) > 0xF;
        self.reg.cf = ((a as u16) + (value as u16) + (carry as u16)) > 0xFF;
    }

    pub fn sub_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let result = self.reg.a.wrapping_sub(value);

        self.reg.a = result;

        self.reg.zf = result == 0;
        self.reg.nf = true;
        self.reg.hf = (a & 0xF) < (value & 0xF);
        self.reg.cf = (a as u16) < (value as u16);
    }

    pub fn sbc_r8(&mut self, value: u8) {
        let a = self.reg.a;
        let carry = self.reg.cf as u8;
        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.reg.a = result;

        self.reg.zf = result == 0;
        self.reg.nf = true;
        self.reg.hf = (a & 0xF) < ((value & 0xF) + (carry & 0xF));
        self.reg.cf = (a as u16) < ((value as u16) + (carry as u16));
    }

    pub fn and_r8(&mut self, value: u8) {
        self.reg.a &= value;

        self.reg.zf = self.reg.a == 0;
        self.reg.nf = false;
        self.reg.hf = true;
        self.reg.cf = false;
    }

    pub fn xor_r8(&mut self, value: u8) {
        self.reg.a ^= value;

        self.reg.zf = self.reg.a == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = false;
    }

    pub fn or_r8(&mut self, value: u8) {
        self.reg.a |= value;

        self.reg.zf = self.reg.a == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = false;
    }

    pub fn cp_r8(&mut self, value: u8) {
        let result = self.reg.a.wrapping_sub(value);

        self.reg.zf = result == 0;
        self.reg.nf = true;
        self.reg.hf = (self.reg.a & 0xF) < (value & 0xF);
        self.reg.cf = (self.reg.a as u16) < (value as u16);
    }

    pub fn unconditional_ret(&mut self, bus: &mut Bus) {
        let lower = self.read_byte(bus, self.reg.sp);
        self.reg.sp += 1;

        let upper = self.read_byte(bus, self.reg.sp);
        self.reg.sp += 1;

        let return_address = u16::from_le_bytes([lower, upper]);

        self.reg.pc = return_address;
        self.internal_cycle(bus);
    }

    pub fn conditional_ret(&mut self, bus: &mut Bus, condition: u8) {
        self.internal_cycle(bus);

        if self.reg.check_condition(condition) {
            self.unconditional_ret(bus);
        }
    }

    pub fn ld_io_u8_a(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as u16;

        self.write_byte(bus, 0xFF00u16.wrapping_add(offset), self.reg.a);
    }

    pub fn add_sp_i8(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16 as u16;
        let sp = self.reg.sp;

        self.reg.sp = sp.wrapping_add(offset);

        self.internal_cycle(bus);
        self.internal_cycle(bus);

        self.reg.zf = false;
        self.reg.nf = false;
        self.reg.hf = ((offset & 0xF) + (sp & 0xF)) > 0xF;
        self.reg.cf = ((offset & 0xFF) + (sp & 0xFF)) > 0xFF;
    }

    pub fn ld_a_io_u8(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as u16;

        self.reg.a = self.read_byte(bus, 0xFF00u16.wrapping_add(offset));
    }

    pub fn ld_hl_sp_i8(&mut self, bus: &mut Bus) {
        let offset = self.imm_byte(bus) as i8 as i16 as u16;
        let sp = self.reg.sp;

        self.reg.set_hl(sp.wrapping_add(offset));
        self.internal_cycle(bus);

        self.reg.zf = false;
        self.reg.nf = false;
        self.reg.hf = ((offset & 0xF) + (sp & 0xF)) > 0xF;
        self.reg.cf = ((offset & 0xFF) + (sp & 0xFF)) > 0xFF;
    }

    pub fn pop_r16(&mut self, bus: &mut Bus, r16: u8) {
        let lower = self.read_byte(bus, self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let upper = self.read_byte(bus, self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let value = u16::from_le_bytes([lower, upper]);
        self.write_r16::<3>(r16, value);
    }

    pub fn reti(&mut self, bus: &mut Bus) {
        self.unconditional_ret(bus);
        self.ime = true;
    }

    pub fn jp_hl(&mut self) {
        self.reg.pc = self.reg.get_hl();
    }

    pub fn ld_sp_hl(&mut self, bus: &mut Bus) {
        self.reg.sp = self.reg.get_hl();
        self.internal_cycle(bus);
    }

    pub fn unconditional_jp(&mut self, bus: &mut Bus) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        let jump_address = u16::from_le_bytes([lower, upper]);

        self.reg.pc = jump_address;
        self.internal_cycle(bus);
    }

    pub fn conditional_jp(&mut self, bus: &mut Bus, condition: u8) {
        let lower = self.imm_byte(bus);
        let upper = self.imm_byte(bus);

        let jump_address = u16::from_le_bytes([lower, upper]);

        if self.reg.check_condition(condition) {
            self.reg.pc = jump_address;
            self.internal_cycle(bus);
        }
    }

    pub fn rlc_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.rotate_left(1);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x80) != 0;
    }

    pub fn rrc_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value.rotate_right(1);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x01) != 0;
    }

    pub fn rl_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let carry = self.reg.cf as u8;
        let result = (value << 1) | carry;

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x80) != 0;
    }

    pub fn rr_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let carry = self.reg.cf as u8;
        let result = (value >> 1) | (carry << 7);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x01) != 0;
    }

    pub fn sla_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value << 1;

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x80) != 0;
    }

    pub fn sra_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = (value >> 1) | (value & 0x80);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x01) != 0;
    }

    pub fn swap_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = (value << 4) | (value >> 4);

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = false;
    }

    pub fn srl_r8(&mut self, bus: &mut Bus, r8: u8) {
        let value = self.read_r8(bus, r8);
        let result = value >> 1;

        self.write_r8(bus, r8, result);

        self.reg.zf = result == 0;
        self.reg.nf = false;
        self.reg.hf = false;
        self.reg.cf = (value & 0x01) != 0;
    }

    pub fn bit_r8(&mut self, bus: &mut Bus, r8: u8, bit: u8) {
        let value = self.read_r8(bus, r8) & (1 << bit);

        self.reg.zf = value == 0;
        self.reg.nf = false;
        self.reg.hf = true;
    }

    pub fn res_r8(&mut self, bus: &mut Bus, r8: u8, bit: u8) {
        let value = self.read_r8(bus, r8);
        let mask = !(1 << bit);
        let result = value & mask;

        self.write_r8(bus, r8, result);
    }

    pub fn set_r8(&mut self, bus: &mut Bus, r8: u8, bit: u8) {
        let value = self.read_r8(bus, r8) | (1 << bit);

        self.write_r8(bus, r8, value);
    }

    pub fn conditional_call(&mut self, bus: &mut Bus, condition: u8) {
        let lower_imm = self.imm_byte(bus);
        let upper_imm = self.imm_byte(bus);

        let address = u16::from_le_bytes([lower_imm, upper_imm]);

        if self.reg.check_condition(condition) {
            self.internal_cycle(bus);

            let [lower, upper] = self.reg.pc.to_le_bytes();

            self.reg.sp = self.reg.sp.wrapping_sub(1);
            self.write_byte(bus, self.reg.sp, upper);

            self.reg.sp = self.reg.sp.wrapping_sub(1);
            self.write_byte(bus, self.reg.sp, lower);

            self.reg.pc = address;
        }
    }

    pub fn unconditional_call(&mut self, bus: &mut Bus) {
        let lower_imm = self.imm_byte(bus);
        let upper_imm = self.imm_byte(bus);

        let address = u16::from_le_bytes([lower_imm, upper_imm]);

        self.internal_cycle(bus);

        let [lower, upper] = self.reg.pc.to_le_bytes();

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.write_byte(bus, self.reg.sp, upper);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.write_byte(bus, self.reg.sp, lower);

        self.reg.pc = address;
    }

    pub fn push_r16(&mut self, bus: &mut Bus, r16: u8) {
        let value = self.read_r16::<3>(r16);
        let [lower, upper] = value.to_le_bytes();

        self.internal_cycle(bus);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.write_byte(bus, self.reg.sp, upper);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.write_byte(bus, self.reg.sp, lower);
    }
}
