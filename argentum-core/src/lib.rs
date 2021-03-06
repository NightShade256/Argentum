#![no_std]

extern crate alloc;

mod bus;
mod cpu;
mod gameboy;
mod timers;

pub use gameboy::GameBoy;
