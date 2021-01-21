//! Contains the main entry point of this library.

use crate::{bus::Bus, cpu::Cpu};

/// T-cycles to execute per frame.
const CYCLES_PER_FRAME: u32 = (4194304.0 / 59.73) as u32;

/// Wraps all the individual components of the Game Boy.
pub struct GameBoy {
    /// The Sharp SM83 CPU.
    cpu: Cpu,

    /// The memory bus interface.
    bus: Bus,
}

impl GameBoy {
    /// Create a new `GameBoy` instance.
    pub fn new(rom_buffer: &[u8]) -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(rom_buffer),
        }
    }

    /// Skip the Game Boy bootrom.
    pub fn skip_bootrom(&mut self) {
        self.cpu.skip_bootrom();
    }

    /// Get a reference to the rendered framebuffer.
    pub fn get_framebuffer(&self) -> &[u8] {
        self.bus.ppu.framebuffer.as_ref()
    }

    /// Execute one frames worth of instructions.
    /// Call this at a rate of 59.73 Hz.
    pub fn execute_frame(&mut self) {
        let mut cycles = 0;

        while cycles <= CYCLES_PER_FRAME {
            // Execute one CPU opcode.
            let t_elapsed = self.cpu.execute_opcode(&mut self.bus);

            // Tick the other components on the bus by
            // the t_elapsed T-cycles.
            self.bus.tick_components(t_elapsed);

            cycles += t_elapsed;
        }
    }
}
