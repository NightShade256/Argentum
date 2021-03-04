use alloc::boxed::Box;

use crate::timers::Timers;

pub struct Bus {
    timers: Timers,
    memory: Box<[u8; 0x10000]>,

    pub if_flag: u8,

    pub ie_flag: u8,
}

impl Bus {
    /// Create a new `Bus` instance.
    pub fn new(rom: &[u8]) -> Self {
        let mut bus = Self {
            timers: Timers::new(),
            memory: Box::new([0; 0x10000]),
            ie_flag: 0,
            if_flag: 0,
        };

        bus.memory[0x0000..=0x7FFF].copy_from_slice(rom);

        bus
    }

    /// Read a byte from the given address.
    pub fn read_byte(&mut self, addr: u16) -> u8 {
        self.tick();
        let value = match addr {
            0xFF04..=0xFF07 => self.timers.read_byte(addr),
            0xFF0F => self.if_flag,
            0xFF44 => 0x90,
            0xFFFF => self.ie_flag,

            _ => self.memory[addr as usize],
        };



        value
    }

    /// Write a byte to the given address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.tick();
        match addr {
            0xFF04..=0xFF07 => self.timers.write_byte(addr, value),
            0xFF0F => self.if_flag = value,
            0xFF44 => {}
            0xFFFF => self.ie_flag = value,

            _ => self.memory[addr as usize] = value,
        }


    }

    /// Tick the components on the Bus.
    pub fn tick(&mut self) {
        self.timers.tick(&mut self.if_flag);
    }
}
