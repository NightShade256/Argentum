//! Contains an implementation of the Sharp SM83 CPU
//! found inside the Game Boy.

mod registers;

use self::registers::*;
use crate::bus::Bus;

/// Implementation of the Sharp SM83.
pub struct Cpu {
    /// Set of all the registers.
    r: Registers,

    /// Interrupt Master Enable switch.
    ime: bool,

    /// Program counter.
    pc: u16,
}

impl Cpu {
    /// Create a new `Cpu` instance.
    pub fn new() -> Self {
        Self {
            r: Registers::new(),
            ime: false,
            pc: 0x0000,
        }
    }

    /// Read a byte from the address pointed to by `PC`.
    pub fn imm_byte(&mut self, bus: &mut Bus) -> u8 {
        let value = bus.read_byte(self.pc);
        self.pc += 1;

        value
    }

    /// Read a word from the address pointed to by `PC`.
    pub fn imm_word(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.pc);
        self.pc += 2;

        value
    }
}
