//! Contains an implementation of the Sharp SM83 CPU
//! found inside the Game Boy.

mod registers;

use registers::*;

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
}
