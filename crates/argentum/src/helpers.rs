pub trait BitExt {
    /// Test a bit of an unsigned integer.
    fn bit(&self, bit: u32) -> bool;

    /// Set a bit of an unsigned integer.
    fn set(&mut self, bit: u32);

    /// Reset a bit of an unsigned integer.
    fn res(&mut self, bit: u32);
}

macro_rules! impl_bit_ext {
    ($($ty:ty),+) => {
        $(impl BitExt for $ty {
            #[inline]
            fn bit(&self, bit: u32) -> bool {
                (*self & (1 << bit)) != 0
            }

            #[inline]
            fn set(&mut self, bit: u32) {
                *self |= (1 << bit);
            }

            #[inline]
            fn res(&mut self, bit: u32) {
                *self &= !(1 << bit);
            }
        })+
    };
}

impl_bit_ext!(u8, u16, u32, u64, u128);
