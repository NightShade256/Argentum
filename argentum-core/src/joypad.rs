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
    /// 0xFF00 - JOYP register.
    ///
    /// Contains the current state of the buttons and DPAD.
    joyp: u8,

    /// Indicates if the DPAD control bit selected.
    dpad: bool,

    /// Indicates if the button control bit selected.
    buttons: bool,

    /// Indicates if a joypad IRQ was triggered.
    irq: bool,
}

impl Joypad {
    /// Create a new `Joypad` instance.
    pub fn new() -> Self {
        Self {
            joyp: 0x00,
            dpad: false,
            buttons: false,
            irq: false,
        }
    }

    /// Request a joypad interrupt if a key previously
    /// went from high to low.
    pub fn tick(&mut self, if_reg: &mut u8) {
        if self.irq {
            *if_reg |= 0b0001_0000;
            self.irq = false;
        }
    }

    /// Register a key being pressed.
    #[inline]
    pub fn key_down(&mut self, key: ArgentumKey) {
        self.joyp |= key as u8;
        self.irq = true;
    }

    /// Register a key being unpressed.
    #[inline]
    pub fn key_up(&mut self, key: ArgentumKey) {
        self.joyp &= !(key as u8);
    }

    /// Read a byte from the specified address.
    pub fn read_byte(&self, _: u16) -> u8 {
        let mut joyp = 0x00;

        joyp |= (self.dpad as u8) << 4;
        joyp |= (self.buttons as u8) << 5;

        if self.dpad {
            joyp |= (self.joyp & 0x0F) >> 0;
        }

        if self.buttons {
            joyp |= (self.joyp & 0xF0) >> 4;
        }

        !joyp
    }

    /// Write a byte to the specified address.
    pub fn write_byte(&mut self, _: u16, value: u8) {
        self.dpad = (value & 0x10) == 0;
        self.buttons = (value & 0x20) == 0;
    }
}
