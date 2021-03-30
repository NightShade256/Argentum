//! Wrapper struct to conviniently abstract the inner workings.

use alloc::boxed::Box;

use crate::{bus::Bus, cpu::Cpu, joypad::GbKey};

/// T-cycles to execute per frame.
const CYCLES_PER_FRAME: u32 = 70224;

pub struct GameBoy {
    bus: Bus,
    cpu: Cpu,
}

impl GameBoy {
    /// Create a new `GameBoy` instance.
    pub fn new(rom: &[u8], callback: Box<dyn Fn(&[f32])>) -> Self {
        Self {
            bus: Bus::new(rom, callback),
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

    /// Get a reference to the framebuffer.
    pub fn get_framebuffer(&self) -> &[u8] {
        self.bus.ppu.front_framebuffer.as_ref()
    }

    pub fn skip_bootrom(&mut self) {
        log::info!("Skipping bootrom, and running game ROM.");

        self.cpu.skip_bootrom(self.bus.cgb_mode);
        self.bus.skip_bootrom();
    }

    /// Redirects to joypad interface.
    pub fn key_down(&mut self, key: GbKey) {
        self.bus.joypad.key_down(key);
    }

    /// Redirects to joypad interface.
    pub fn key_up(&mut self, key: GbKey) {
        self.bus.joypad.key_up(key);
    }
}
