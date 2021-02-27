//! Contains implementation of the Directional Pad and Buttons.

use bitflags::bitflags;

use crate::common::MemInterface;

bitflags! {
    pub struct GbKey: u8 {
        // Directional Keys.
        const RIGHT = 0b0000_0001;
        const LEFT = 0b0000_0010;
        const UP = 0b0000_0100;
        const DOWN = 0b0000_1000;

        // Normal Buttons.
        const BUTTONA = 0b0001_0000;
        const BUTTONB = 0b0010_0000;
        const SELECT = 0b0100_0000;
        const START = 0b1000_0000;
    }
}

pub struct Joypad {
    /// Are directional keys selected?
    directional: bool,

    /// Are normal buttons selected?
    buttons: bool,

    /// Current keypad state.
    state: GbKey,

    /// Should request joypad interrupt?
    irq: bool,
}

impl MemInterface for Joypad {
    fn read_byte(&self, _: u16) -> u8 {
        let mut byte = 0x00;

        // Insert the selection bits.
        byte |= (self.directional as u8) << 4;
        byte |= (self.buttons as u8) << 5;

        if self.buttons {
            byte |= (self.state.contains(GbKey::START) as u8) << 3;
            byte |= (self.state.contains(GbKey::SELECT) as u8) << 2;
            byte |= (self.state.contains(GbKey::BUTTONB) as u8) << 1;
            byte |= self.state.contains(GbKey::BUTTONA) as u8;
        }

        if self.directional {
            byte |= (self.state.contains(GbKey::DOWN) as u8) << 3;
            byte |= (self.state.contains(GbKey::UP) as u8) << 2;
            byte |= (self.state.contains(GbKey::LEFT) as u8) << 1;
            byte |= self.state.contains(GbKey::RIGHT) as u8;
        }

        !byte
    }

    /// We only care about the bits 4 and 5 since other
    /// bits are read-only or are not used.
    fn write_byte(&mut self, _: u16, value: u8) {
        self.directional = (value & 0x10) == 0;
        self.buttons = (value & 0x20) == 0;
    }
}

impl Joypad {
    /// Create a new `Joypad` instance.
    pub fn new() -> Self {
        Self {
            directional: false,
            buttons: false,
            state: GbKey::empty(),
            irq: false,
        }
    }

    /// This is a stub.
    /// This only requests a joypad interrupt, if a button goes from hi to lo.
    pub fn tick(&mut self, _: u32, if_reg: &mut u8) {
        if self.irq {
            *if_reg |= 0b0001_0000;
            self.irq = false;
        }
    }

    /// Register the key being pressed.
    #[inline]
    pub fn key_down(&mut self, key: GbKey) {
        self.state.insert(key);
        self.irq = true;
    }

    /// Register the key being unpressed.
    #[inline]
    pub fn key_up(&mut self, key: GbKey) {
        self.state.remove(key);
    }
}
