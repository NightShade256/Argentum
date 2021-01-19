//! Contains common traits and functionality.

/// Defines an interface for reading and writing bytes and
/// words.
pub trait MemInterface {
    /// Read a byte from the given address.
    fn read_byte(&self, addr: u16) -> u8;

    /// Write a byte to the given address.
    fn write_byte(&mut self, addr: u16, value: u8);

    /// Read a word from the given address.
    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    /// Write a word to the given address.
    fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }
}
