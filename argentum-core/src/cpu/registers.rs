//! Contains helper struct `Registers` with methods
//! to access 16 bit combined versions of the regular
//! 8 bit registers.

use bitflags::bitflags;

use crate::bus::Bus;

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
    pub a: u8,

    // General registers.
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Flag register.
    pub f: Flags,

    // Stack pointer.
    pub sp: u16,

    // Program counter.
    pub pc: u16,
}

/// Enumerates all 16 bit registers.
#[derive(Clone, Copy)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,

    HLI, // HL, post inc.
    HLD, // HL, post dec.
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Reg8 {
    B = 0,
    C,
    D,
    E,
    H,
    L,
    HL, // [HL] aka byte pointed to by HL.
    A,
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
            pc: 0,
        }
    }

    /// Read the value of a 16 bit register.
    pub fn read_r16(&mut self, r16: Reg16) -> u16 {
        use Reg16::*;

        match r16 {
            AF => ((self.a as u16) << 8) | self.f.bits() as u16,
            BC => ((self.b as u16) << 8) | self.c as u16,
            DE => ((self.d as u16) << 8) | self.e as u16,
            HL => ((self.h as u16) << 8) | self.l as u16,
            SP => self.sp,

            HLI => {
                let value = ((self.h as u16) << 8) | self.l as u16;
                self.write_r16(Reg16::HL, value.wrapping_add(1));

                value
            }

            HLD => {
                let value = ((self.h as u16) << 8) | self.l as u16;
                self.write_r16(Reg16::HL, value.wrapping_sub(1));

                value
            }
        }
    }

    /// Write a value to a 16 bit register.
    pub fn write_r16(&mut self, r16: Reg16, value: u16) {
        use Reg16::*;

        match r16 {
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

            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn read_r8(&mut self, r8: Reg8, bus: &Bus) -> u8 {
        use Reg8::*;

        match r8 {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
            A => self.a,

            HL => bus.read_byte(self.read_r16(Reg16::HL)),
        }
    }

    #[inline]
    pub fn write_r8(&mut self, r8: Reg8, bus: &mut Bus, value: u8) {
        use Reg8::*;

        match r8 {
            B => self.b = value,
            C => self.c = value,
            D => self.d = value,
            E => self.e = value,
            H => self.h = value,
            L => self.l = value,
            A => self.a = value,

            HL => bus.write_byte(self.read_r16(Reg16::HL), value),
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
