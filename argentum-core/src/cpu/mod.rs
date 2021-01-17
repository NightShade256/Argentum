//! Contains an implementation of the Sharp SM83 CPU
//! found inside the Game Boy.

use std::fmt::*;

mod decode;
mod instructions;
mod registers;

use self::registers::*;
use crate::bus::Bus;

/// Implementation of the Sharp SM83.
pub struct Cpu {
    /// Set of all the registers.
    r: Registers,

    /// Interrupt Master Enable switch.
    ime: bool,
}

/// Formatted similar to wheremyfoodat's (peach's) logs.
impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let reg_one = format!(
            "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X}",
            self.r.a,
            self.r.f.bits(),
            self.r.b,
            self.r.c,
            self.r.d
        );

        let reg_two = format!(
            "E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: {:04X}",
            self.r.e, self.r.h, self.r.l, self.r.sp, self.r.pc,
        );

        write!(f, "{} {}", reg_one, reg_two)
    }
}

impl Cpu {
    /// Create a new `Cpu` instance.
    pub fn new() -> Self {
        Self {
            r: Registers::new(),
            ime: false,
        }
    }

    /// Skips the bootrom, and initializes default values for
    /// registers.
    pub fn skip_bootrom(&mut self) {
        self.r.write_r16(Reg16::AF, 0x01B0);
        self.r.write_r16(Reg16::BC, 0x0013);
        self.r.write_r16(Reg16::DE, 0x00D8);
        self.r.write_r16(Reg16::HL, 0x014D);

        self.r.sp = 0xFFFE;
        self.r.pc = 0x0100;
    }

    /// Read a byte from the address pointed to by `PC`.
    pub fn imm_byte(&mut self, bus: &mut Bus) -> u8 {
        let value = bus.read_byte(self.r.pc);
        self.r.pc += 1;

        value
    }

    /// Read a word from the address pointed to by `PC`.
    pub fn imm_word(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.r.pc);
        self.r.pc += 2;

        value
    }
}
