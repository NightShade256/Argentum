//! Implementation of the Sharp SM83 CPU core.

mod decode;
mod instructions;
mod registers;

use self::registers::Registers;
use crate::bus::Bus;

/// Enumerates all the states the CPU can be in.
pub enum CpuState {
    Halted,
    Running,
}

pub struct CPU {
    // All the registers associated with the CPU.
    reg: Registers,

    // The Interrupt Master Enable flag.
    // Interrupts are serviced iff this flag is enabled.
    ime: bool,

    // The state the CPU is in.
    state: CpuState,
}

impl CPU {
    /// Create a new `CPU` instance.
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            ime: false,
            state: CpuState::Running,
        }
    }

    /// Read a byte from the current PC address.
    pub fn imm_byte(&mut self, bus: &Bus) -> u8 {
        let value = bus.read_byte(self.reg.pc);
        self.reg.pc += 1;

        value
    }

    /// TODO
    /// Tick components by one M cycle.
    pub fn internal_cycle(&self, _bus: &mut Bus) {}

    /// Read a R16 by specifiying the group and its index.
    /// See wheremyfoodat's decoding opcode PDF.
    pub fn read_r16<const GROUP: u8>(&mut self, r16: u8) -> u16 {
        match GROUP {
            1 => match r16 {
                0 => self.reg.get_bc(),
                1 => self.reg.get_de(),
                2 => self.reg.get_hl(),
                3 => self.reg.sp,

                _ => unreachable!(),
            },

            2 => match r16 {
                0 => self.reg.get_bc(),
                1 => self.reg.get_de(),
                2 => {
                    let value = self.reg.get_hl();
                    self.reg.set_hl(value.wrapping_add(1));

                    value
                }
                3 => {
                    let value = self.reg.get_hl();
                    self.reg.set_hl(value.wrapping_sub(1));

                    value
                }

                _ => unreachable!(),
            },

            3 => match r16 {
                0 => self.reg.get_bc(),
                1 => self.reg.get_de(),
                2 => self.reg.get_hl(),
                3 => self.reg.get_af(),

                _ => unreachable!(),
            },

            _ => unreachable!(),
        }
    }

    /// Write a value to a R16 by specifiying the group and its index.
    /// See wheremyfoodat's decoding opcode PDF.
    pub fn write_r16<const GROUP: u8>(&mut self, r16: u8, value: u16) {
        match GROUP {
            1 => match r16 {
                0 => self.reg.set_bc(value),
                1 => self.reg.set_de(value),
                2 => self.reg.set_hl(value),
                3 => self.reg.sp = value,

                _ => unreachable!(),
            },

            2 => match r16 {
                0 => self.reg.set_bc(value),
                1 => self.reg.set_de(value),
                2 | 3 => self.reg.set_hl(value),

                _ => unreachable!(),
            },

            3 => match r16 {
                0 => self.reg.set_bc(value),
                1 => self.reg.set_de(value),
                2 => self.reg.set_hl(value),
                3 => self.reg.set_af(value),

                _ => unreachable!(),
            },

            _ => unreachable!(),
        }
    }
}
