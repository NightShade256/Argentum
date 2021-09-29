use crate::bus::Bus;
use crate::cpu::Cpu;

#[derive(Debug, Default)]
pub struct Registers {
    // General Purpose Registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Flags
    pub zf: bool,
    pub nf: bool,
    pub hf: bool,
    pub cf: bool,

    // Stack Pointer and Program Counter
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    /// Create a new `Registers` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the given condition is true.
    pub fn check_condition(&self, condition: u8) -> bool {
        match condition {
            0 => !self.zf,
            1 => self.zf,
            2 => !self.cf,
            3 => self.cf,

            _ => unreachable!(),
        }
    }

    /* 16-bit register getters */

    #[inline]
    pub fn get_af(&self) -> u16 {
        let self_f = ((self.zf as u8) << 7)
            | ((self.nf as u8) << 6)
            | ((self.hf as u8) << 5)
            | ((self.cf as u8) << 4);

        ((self.a as u16) << 8) | self_f as u16
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

    /* 16-bit register setters */

    #[inline]
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;

        self.zf = ((value >> 7) & 0x01) != 0;
        self.nf = ((value >> 6) & 0x01) != 0;
        self.hf = ((value >> 5) & 0x01) != 0;
        self.cf = ((value >> 4) & 0x01) != 0;
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
}

impl Cpu {
    /// Read the value of a 8-bit register by specifiying its group and index.
    pub fn read_r8(&mut self, bus: &mut Bus, r8: u8) -> u8 {
        match r8 {
            0 => self.reg.b,
            1 => self.reg.c,
            2 => self.reg.d,
            3 => self.reg.e,
            4 => self.reg.h,
            5 => self.reg.l,
            6 => self.read_byte(bus, self.reg.get_hl()),
            7 => self.reg.a,

            _ => unreachable!(),
        }
    }

    /// Write a value to a 8-bit register by specifiying its group and index.
    pub fn write_r8(&mut self, bus: &mut Bus, r8: u8, value: u8) {
        match r8 {
            0 => self.reg.b = value,
            1 => self.reg.c = value,
            2 => self.reg.d = value,
            3 => self.reg.e = value,
            4 => self.reg.h = value,
            5 => self.reg.l = value,
            6 => self.write_byte(bus, self.reg.get_hl(), value),
            7 => self.reg.a = value,

            _ => unreachable!(),
        }
    }

    /// Read the value of a 16-bit register by specifiying its group and index.
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

    /// Write a value to a 16-bit register by specifiying its group and index.
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
