use crate::bus::Bus;
use crate::helpers::BitExt;

impl Bus {
    /// Returns `true` if the system is currently in CGB
    /// double speed mode.
    pub fn is_double_speed(&self) -> bool {
        self.key1.bit(7)
    }

    /// Returns `true` if the system is currently preparing
    /// to switch speed mode.
    pub fn is_preparing_switch(&self) -> bool {
        self.key1.bit(0)
    }

    /// Switches the current speed mode to the other.
    /// CGB Double Speed -> Normal Speed
    /// Normal Speed     -> CGB Double Speed
    pub fn perform_speed_switch(&mut self) {
        if self.is_double_speed() {
            self.key1.res(7);
        } else {
            self.key1.set(7);
        }

        self.key1.res(0);
    }
}
