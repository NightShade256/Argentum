use crate::helpers::set;

/// Enumerates all possible keys that are present on the
/// Game Boy and Game Boy Color.
#[repr(u8)]
pub enum ArgentumKey {
    Right = 0x01,
    Left = 0x02,
    Up = 0x04,
    Down = 0x08,
    ButtonA = 0x10,
    ButtonB = 0x20,
    Select = 0x40,
    Start = 0x80,
}

#[derive(Default)]
pub struct Joypad {
    /// Indicates if the buttons control bit selected.
    buttons: bool,

    /// Indicates if the DPAD control bit selected.
    dpad: bool,

    /// Indicates if a joypad interrupt should be requested.
    interrupt_requested: bool,

    /// Contains the current state of the buttons and DPAD.
    joypad_state: u8,
}

impl Joypad {
    /// Create a new `Joypad` instance.
    pub fn new() -> Self {
        Self {
            buttons: true,
            dpad: true,
            ..Default::default()
        }
    }

    /// Check if an interrupt is requested, and if yes set the joypad bit
    /// in the provided reference to IF register.
    pub fn update_interrupt_state(&mut self, if_reg: &mut u8) {
        if self.interrupt_requested {
            set!(if_reg, 4);
        }

        self.interrupt_requested = false;
    }

    /// Register a particular key as being pressed.
    pub fn key_down(&mut self, key: ArgentumKey) {
        self.joypad_state |= key as u8;
        self.interrupt_requested = true;
    }

    /// Register a particular key as being released.
    pub fn key_up(&mut self, key: ArgentumKey) {
        self.joypad_state &= !(key as u8);
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        if addr == 0xFF00 {
            let mut joyp = 0x00;

            joyp |= (self.dpad as u8) << 4;
            joyp |= (self.buttons as u8) << 5;

            if self.dpad {
                joyp |= (self.joypad_state & 0x0F) >> 0;
            }

            if self.buttons {
                joyp |= (self.joypad_state & 0xF0) >> 4;
            }

            !joyp
        } else {
            unreachable!()
        }
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        if addr == 0xFF00 {
            self.dpad = (value & 0x10) == 0;
            self.buttons = (value & 0x20) == 0;
        } else {
            unreachable!()
        }
    }
}
