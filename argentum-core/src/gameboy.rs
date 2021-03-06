//! Wrapper struct to conviniently abstract the inner workings.

use crate::bus::Bus;
use crate::cpu::Cpu;

/// T-cycles to execute per frame.
const CYCLES_PER_FRAME: u32 = (4194304.0 / 59.73) as u32;

pub struct GameBoy {
    bus: Bus,
    cpu: Cpu,
}

impl GameBoy {
    /// Create a new `GameBoy` instance.
    pub fn new(rom: &[u8]) -> Self {
        Self {
            bus: Bus::new(rom),
            cpu: Cpu::new(),
        }
    }

    /// Execute a frame's worth of instructions.
    pub fn execute_frame(&mut self) {
        let mut cycles = 0;

        while cycles <= CYCLES_PER_FRAME {
            cycles += self.cpu.execute_next(&mut self.bus);
        }
    }
}
