use crate::helpers::set;

#[derive(Default)]
pub(crate) struct Timer {
    /// 0xFF04 - Divider Register.
    ///
    /// It is incremented every T-cycle, but only the upper
    /// 8 bits are mapped to memory, thus giving it the appearance of
    /// being incremented every 256 T-cycles.
    div: u16,

    /// 0xFF05 - Timer Counter.
    ///
    /// This is a configurable timer, which can be enabled or disabled
    /// and whose frequency can be changed.
    tima: u8,

    /// 0xFF06 - Timer Modulo.
    ///
    /// Whenever the TIMA timer overflows, the value stored in this
    /// register is loaded into TIMA.
    tma: u8,

    /// 0xFF07 - Timer Control.
    ///
    /// This register controls the frequency of TIMA, and also controls
    /// whether TIMA is incremented or not.
    tac: u8,

    /// Stores the last AND Result, used to detect falling edge on the
    /// selected bit of DIV.
    last_and_result: u8,

    /// The T-cycles remaining for TIMA reload to occur, if any.
    tima_reload: Option<u8>,
}

impl Timer {
    /// Create a new `Timer` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Tick the timers and divider by 4 T-cycles.
    pub fn tick(&mut self, if_reg: &mut u8, cycles: u32) {
        for _ in 0..cycles {
            if let Some(ref mut cycles) = self.tima_reload {
                if *cycles == 0 {
                    self.tima_reload = None;
                } else {
                    *cycles -= 4;

                    if *cycles == 0 {
                        self.tima = self.tma;
                        set!(if_reg, 2);
                    }
                }
            }

            self.div = self.div.wrapping_add(4);
            self.check_falling_edge();
        }
    }

    /// Check for a falling edge on the selected bit of DIV.
    fn check_falling_edge(&mut self) {
        let bit = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,

            _ => unreachable!(),
        };

        let and_result = (((self.div >> bit) & 0x01) as u8) & ((self.tac >> 2) & 0x01);

        if (self.last_and_result & !and_result) != 0 {
            let (result, overflow) = self.tima.overflowing_add(1);
            self.tima = result;

            if overflow {
                self.tima_reload = Some(4);
            }
        }

        self.last_and_result = and_result;
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,

            _ => unreachable!(),
        }
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.div = 0x00,

            0xFF05 => {
                if self.tima_reload != Some(0) {
                    self.tima = value;
                    self.tima_reload = None;
                }
            }

            0xFF06 => {
                self.tma = value;

                if self.tima_reload == Some(0) {
                    self.tima = self.tma;
                }
            }

            0xFF07 => {
                self.tac = value & 0x07;
                self.check_falling_edge();
            }

            _ => unreachable!(),
        }
    }
}
