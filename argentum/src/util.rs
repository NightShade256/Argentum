/// Get a bit of a number.
macro_rules! get_bit {
    ($number:expr, $bit:expr) => {
        (($number & (1 << $bit)) != 0)
    };
}

/// Set a bit of a number.
macro_rules! set_bit {
    ($number:expr, $bit:expr) => {
        *($number) |= (1 << $bit);
    };
}

/// Reset a bit of a number.
macro_rules! res_bit {
    ($number:expr, $bit:expr) => {
        *($number) &= !(1 << $bit);
    };
}

pub(crate) use {get_bit, res_bit, set_bit};
