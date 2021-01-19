//! Implementation of the Game Boy timer system.

use crate::common::MemInterface;

/// All the frequencies at which TIMA can tick.
const TIMA_FREQUENCY: [u32; 4] = [1024, 16, 64, 256];

pub struct Timers {
    /// T-cycles elapsed since DIV was incremented.
    div_cycles: u32,

    /// T-cycles elapsed since TIMA was incremented.
    tima_cycles: u32,

    /// Incremented every 256 T-cycles (16384 Hz)
    div: u8,

    /// Incremented at a rate specified by TAC.
    tima: u8,

    /// The value loaded into TIMA when TIMA overflows.
    tma: u8,

    /// Timer Control
    tac: u8,
}

impl MemInterface for Timers {
    /// Read a byte from the given address.
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,

            _ => unreachable!(),
        }
    }

    /// Write a byte to the given address.
    fn write_byte(&mut self, addr: u16, value: u8) {
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

impl Timers {
    /// Create a new `Timer` instance.
    pub fn new() -> Self {
        Self {
            div_cycles: 0,
            tima_cycles: 0,
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    /// Tick the timers.
    /// Should be called after every opcode.
    pub fn tick(&mut self, t_elapsed: u32, if_reg: &mut u8) {
        // Check if TIMA is enabled.
        if (self.tac & 0x04) != 0 {
            // Get the configured frequency at which TIMA
            // is supposed to be incremented.
            let tima_freq = TIMA_FREQUENCY[(self.tac & 0x03) as usize];

            self.tima_cycles += t_elapsed;

            // If more than the required cycles have passed
            // increment TIMA.
            while self.tima_cycles >= tima_freq {
                let (result, overflow) = self.tima.overflowing_add(1);

                // If there was an overflow, set TIMA = TMA
                // and request a timer interrupt by setting appropriate
                // byte in IF.
                if overflow {
                    self.tima = self.tma;
                    *if_reg |= 0b0000_0100;
                } else {
                    self.tima = result;
                }

                self.tima_cycles -= tima_freq;
            }
        }

        self.div_cycles += t_elapsed;

        // If more than 256 T-cycles have elapsed since
        // DIV was last incremented, increment DIV.
        while self.div_cycles >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_cycles -= 256;
        }
    }
}
