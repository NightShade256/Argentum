use std::{cell::RefCell, rc::Rc};

use crate::util::set;

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

pub struct Joypad {
    /// Contains the current state of the buttons and DPAD.
    joypad_state: u8,

    /// Indicates if the DPAD control bit selected.
    dpad: bool,

    /// Indicates if the button control bit selected.
    buttons: bool,

    /// Shared reference to IF register.
    if_reg: Rc<RefCell<u8>>,
}

impl Joypad {
    /// Create a new `Joypad` instance.
    pub fn new(if_reg: Rc<RefCell<u8>>) -> Self {
        Self {
            joypad_state: 0x00,
            dpad: false,
            buttons: false,
            if_reg,
        }
    }

    /// Register a key being pressed.
    pub fn key_down(&mut self, key: ArgentumKey) {
        self.joypad_state |= key as u8;
        set!(self.if_reg.borrow_mut(), 4);
    }

    /// Register a key being unpressed.
    pub fn key_up(&mut self, key: ArgentumKey) {
        self.joypad_state &= !(key as u8);
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, _: u16) -> u8 {
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
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, _: u16, value: u8) {
        self.dpad = (value & 0x10) == 0;
        self.buttons = (value & 0x20) == 0;
    }
}
