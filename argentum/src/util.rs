/// Get a bit of a number.
macro_rules! bit {
    ($num:expr, $bit:expr) => {
        (*($num) & (1 << $bit)) != 0
    };
}

/// Set a bit of a number.
macro_rules! set {
    ($num:expr, $bit:expr) => {
        *($num) |= (1 << $bit);
    };
}

/// Reset a bit of a number.
macro_rules! res {
    ($num:expr, $bit:expr) => {
        *($num) &= !(1 << $bit);
    };
}

pub(crate) use {bit, res, set};
