use crate::bus::Bus;
use crate::helpers::{bit, res, set};

impl Bus {
    /// Returns `true` if the system is currently in CGB
    /// double speed mode.
    pub fn is_double_speed(&self) -> bool {
        bit!(&self.key1, 7)
    }

    /// Returns `true` if the system is currently preparing
    /// to switch speed mode.
    pub fn is_preparing_switch(&self) -> bool {
        bit!(&self.key1, 0)
    }

    /// Switches the current speed mode to the other.
    /// CGB Double Speed -> Normal Speed
    /// Normal Speed     -> CGB Double Speed
    pub fn perform_speed_switch(&mut self) {
        if self.is_double_speed() {
            res!(&mut self.key1, 7);
        } else {
            set!(&mut self.key1, 7);
        }

        res!(&mut self.key1, 0);
    }
}
