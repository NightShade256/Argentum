//! Wrapper struct to conviniently abstract the inner workings.

use crate::{bus::Bus, cpu::Cpu, joypad::ArgentumKey};

/// T-cycles to execute per frame.
const CYCLES_PER_FRAME: u32 = 70224;

pub struct Argentum {
    bus: Bus,
    cpu: Cpu,
}

impl Argentum {
    /// Create a new `Argentum` instance.
    pub fn new(rom: &[u8], callback: Box<dyn Fn(&[f32])>, save_file: Option<Vec<u8>>) -> Self {
        let mut argentum = Self {
            bus: Bus::new(rom, callback, save_file),
            cpu: Cpu::new(),
        };

        if argentum.bus.cgb_mode {
            argentum.cpu.skip_bootrom(argentum.bus.cgb_mode);
            argentum.bus.skip_bootrom();
        }

        argentum
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
        if self.bus.cgb_mode {
            return;
        }

        self.cpu.skip_bootrom(self.bus.cgb_mode);
        self.bus.skip_bootrom();
    }

    /// Redirects to joypad interface.
    pub fn key_down(&mut self, key: ArgentumKey) {
        self.bus.joypad.key_down(key);
    }

    /// Redirects to joypad interface.
    pub fn key_up(&mut self, key: ArgentumKey) {
        self.bus.joypad.key_up(key);
    }

    /// Dump the SRAM and get a copy.
    pub fn get_ram_dump(&self) -> Option<Vec<u8>> {
        if !([0x03, 0x0F, 0x10, 0x13, 0x1B, 0x1E].contains(&self.bus.cartridge.read_byte(0x0147))) {
            return None;
        }

        self.bus.cartridge.dump_ram()
    }
}
