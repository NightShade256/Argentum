/// Get a bit of a number.
#[macro_export]
macro_rules! get_bit {
    ($number:expr, $bit:expr) => {
        (($number & (1 << $bit)) != 0)
    };
}

/// Set a bit of a number to the given value.
#[macro_export]
macro_rules! set_bit {
    ($number:expr, $bit:expr, $value:expr) => {
        if $value {
            *($number) |= (1 << $bit);
        } else {
            *($number) &= !(1 << $bit);
        }
    };
}
