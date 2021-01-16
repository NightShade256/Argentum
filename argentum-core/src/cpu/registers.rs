//! Contains helper struct `Registers` with methods
//! to access 16 bit combined versions of the regular
//! 8 bit registers.

use bitflags::bitflags;

bitflags! {
    pub struct Flags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

pub struct Registers {
    // Accumulator.
    a: u8,

    // General registers.
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    // Flag register.
    f: Flags,

    // Stack pointer.
    sp: u16,
}

/// Enumerates all 16 bit registers.
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Registers {
    /// Create a new empty `Registers` instance.
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            f: Flags::empty(),
            sp: 0,
        }
    }

    /// Read the value of a 16 bit register.
    pub fn read_rr(&self, reg: Reg16) -> u16 {
        use Reg16::*;

        match reg {
            AF => ((self.a as u16) << 8) | self.f.bits() as u16,
            BC => ((self.b as u16) << 8) | self.c as u16,
            DE => ((self.d as u16) << 8) | self.e as u16,
            HL => ((self.h as u16) << 8) | self.l as u16,
            SP => self.sp,
        }
    }

    /// Write a value to a 16 bit register.
    pub fn write_rr(&mut self, reg: Reg16, value: u16) {
        use Reg16::*;

        match reg {
            AF => {
                self.a = (value >> 8) as u8;
                self.f = Flags::from_bits_truncate(value as u8);
            }

            BC => {
                self.b = (value >> 8) as u8;
                self.c = value as u8;
            }

            DE => {
                self.d = (value >> 8) as u8;
                self.e = value as u8;
            }

            HL => {
                self.h = (value >> 8) as u8;
                self.l = value as u8;
            }

            SP => self.sp = value,
        }
    }

    #[inline]
    pub fn get_zf(&self) -> bool {
        self.f.contains(Flags::Z)
    }

    #[inline]
    pub fn set_zf(&mut self, value: bool) {
        self.f.set(Flags::Z, value);
    }

    #[inline]
    pub fn get_nf(&self) -> bool {
        self.f.contains(Flags::N)
    }

    #[inline]
    pub fn set_nf(&mut self, value: bool) {
        self.f.set(Flags::N, value);
    }

    #[inline]
    pub fn get_hf(&self) -> bool {
        self.f.contains(Flags::H)
    }

    #[inline]
    pub fn set_hf(&mut self, value: bool) {
        self.f.set(Flags::H, value);
    }

    #[inline]
    pub fn get_cf(&self) -> bool {
        self.f.contains(Flags::C)
    }

    #[inline]
    pub fn set_cf(&mut self, value: bool) {
        self.f.set(Flags::C, value);
    }
}
