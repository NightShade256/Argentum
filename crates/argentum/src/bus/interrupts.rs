use super::Bus;

impl Bus {
    /// Get an immutable reference to the IF register.
    pub fn get_if(&self) -> &u8 {
        &self.if_reg
    }

    /// Get an immuatable reference to the IE register.
    pub fn get_ie(&self) -> &u8 {
        &self.ie_reg
    }

    /// Get a mutable reference to the IF register.
    pub fn get_if_mut(&mut self) -> &mut u8 {
        &mut self.if_reg
    }

    /// Get a mutable reference to the IE register.
    #[allow(dead_code)]
    pub fn get_ie_mut(&mut self) -> &mut u8 {
        &mut self.ie_reg
    }
}
