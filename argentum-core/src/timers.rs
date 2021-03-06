//! Implementation of the Game Boy timer sub-system.

pub struct Timers {
    /// 0xFF04 - Divider Register.
    /// It is incremented every T cycle, but only the upper
    /// 8 bits are mapped to memory, thus giving it the appearance of
    /// incrementing every 256 T cycles.
    div: u16,

    /// 0xFF05 - Timer Counter.
    /// This is a configurable timer, which can be enabled or disabled
    /// and whose frequency can be changed.
    tima: u8,

    /// 0xFF06 - Timer Modulo.
    /// Whenever the TIMA timer overflows, the value stored in this
    /// register is loaded into TIMA.
    tma: u8,

    /// 0xFF07 - Timer Control.
    /// This register controls the frequency of TIMA, and also controls
    /// whether TIMA is incremented or not.
    tac: u8,

    /// Stores the last AND Result, used to detect falling edge.
    last_and_result: u8,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            last_and_result: 0,
        }
    }

    /// Tick the timers by 1 M cycle.
    /// 1 M-cycle = 4 T-cycles.
    pub fn tick(&mut self, if_reg: &mut u8) {
        // DIV is incremented every T-cycle.
        self.div = self.div.wrapping_add(4);

        // Select a bit of DIV depending upon the
        // configuration of TAC.
        let bit = match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,

            _ => unreachable!(),
        };

        // Compute the AND result.
        // AND Result = Timer Enable & Selected Bit.
        let and_result = (((self.div >> bit) & 0x01) as u8) & ((self.tac >> 2) & 0x01);

        // Detect a falling edge and increment TIMA.
        if (self.last_and_result & !and_result) != 0 {
            let (result, overflow) = self.tima.overflowing_add(1);

            // TODO: Implement the 1 M cycle delay between overflow
            //       and reloading of TIMA and requesting interrupt.
            if overflow {
                // Reload TIMA with the value in TMA.
                self.tima = self.tma;

                // Request a timer interrupt.
                *if_reg |= 0b0000_0100;
            } else {
                self.tima = result;
            }
        }

        self.last_and_result = and_result;
    }

    /// Read DIV, TIMA, TMA, TAC.
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,

            _ => unreachable!(),
        }
    }

    /// Write to DIV, TIMA, TMA, TAC.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // Attempting to write anything to DIV
            // will result in it being reset to 0x00.
            0xFF04 => self.div = 0,

            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,

            _ => unreachable!(),
        }
    }
}
