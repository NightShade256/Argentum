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
