//! Helper functions for reading, writing registers.

use bitflags::bitflags;

bitflags! {
    pub struct Flags: u8 {
        const Z = 1 << 7;
        const N = 1 << 6;
        const H = 1 << 5;
        const C = 1 << 4;
    }
}

pub struct Registers {
    // Accumulator.
    pub a: u8,

    // General Purpose Registers.
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Flag Register.
    pub f: Flags,

    // Stack Pointer, and Program Counter.
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    /// Create a new `Registers` instance.
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Flags::empty(),
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    #[inline]
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f.bits() as u16
    }

    #[inline]
    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    #[inline]
    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    #[inline]
    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    #[inline]
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = Flags::from_bits_truncate(value as u8);
    }

    #[inline]
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    #[inline]
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    #[inline]
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    #[inline]
    pub fn get_flag(&self, flag: Flags) -> bool {
        self.f.contains(flag)
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Flags, value: bool) {
        self.f.set(flag, value);
    }
}
