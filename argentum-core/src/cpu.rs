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
    r: Registers,

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
            r: Registers::new(),
            ime: false,
            state: CpuState::Running,
        }
    }

    /// Read a byte from the current PC address.
    pub fn imm_byte(&mut self, bus: &Bus) -> u8 {
        let value = bus.read_byte(self.r.pc);
        self.r.pc += 1;

        value
    }

    pub fn internal_cycle(&self, _bus: &mut Bus) {}
}
