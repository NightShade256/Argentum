//! Implementation of the Sharp SM83 CPU core.

mod decode;
mod instructions;
mod registers;

use self::registers::Registers;
use crate::{
    bus::Bus,
    helpers::{bit, res},
};

/// Enumerates all the states the CPU can be in.
#[derive(PartialEq)]
pub enum CpuState {
    Halted,
    Running,
}

/// Implementation of the Sharp SM83 CPU.
pub(crate) struct Cpu {
    /// All the registers associated with the CPU.
    pub reg: Registers,

    /// The Interrupt Master Enable flag.
    /// Interrupts are serviced iff this flag is enabled.
    pub ime: bool,

    /// The state the CPU is in.
    pub state: CpuState,

    /// The amount of cycles spent executing the current
    /// instruction.
    pub cycles: u32,

    /// Are we currently in double speed mode?
    pub is_double_speed: bool,
}

impl Cpu {
    /// Create a new `CPU` instance.
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
            ime: false,
            state: CpuState::Running,
            cycles: 0,
            is_double_speed: false,
        }
    }

    /// A wrapper function over `Bus::read_byte`. This function should be
    /// called and not the other in all CPU instruction handlers.
    pub fn read_byte(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        self.cycles += 4;
        bus.read_byte(addr, true)
    }

    /// A wrapper function over `Bus::write_byte`. This function should be
    /// called and not the other in all CPU instruction handlers.
    pub fn write_byte(&mut self, bus: &mut Bus, addr: u16, value: u8) {
        self.cycles += 4;
        bus.write_byte(addr, value, true);
    }

    /// Execute an internal cycle (used when doing 16-bit arithmetic or jumps).
    pub fn internal_cycle(&mut self, bus: &mut Bus) {
        self.cycles += 4;
        bus.tick(4);
    }

    /// Read a byte from the address contained within the PC, and increment
    /// PC afterwards.
    pub fn imm_byte(&mut self, bus: &mut Bus) -> u8 {
        self.reg.pc = self.reg.pc.wrapping_add(1);
        self.read_byte(bus, self.reg.pc.wrapping_sub(1))
    }

    /// Skips the bootrom, and initializes default values for
    /// registers.
    pub fn skip_bootrom(&mut self, cgb: bool) {
        if cgb {
            self.reg.set_af(0x1180);
            self.reg.set_bc(0x0000);
            self.reg.set_de(0xFF56);
            self.reg.set_hl(0x000D);
        } else {
            self.reg.set_af(0x01B0);
            self.reg.set_bc(0x0013);
            self.reg.set_de(0x00D8);
            self.reg.set_hl(0x014D);
        }

        self.reg.sp = 0xFFFE;
        self.reg.pc = 0x0100;
    }

    /// Handle all pending interrupts.
    /// Only one interrupt is serviced at one time.
    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        let interrupts = bus.ie_reg & bus.if_reg;

        // If there are pending interrupts, CPU should be
        // back up and running.
        if interrupts != 0 {
            self.state = CpuState::Running;
        }

        // If IME is not enabled, we don't service the interrupt.
        if !self.ime {
            return;
        }

        if interrupts != 0 {
            for i in 0..5 {
                if bit!(&bus.ie_reg, i) && bit!(&bus.if_reg, i) {
                    // Disable the interrupt in IF.
                    res!(&mut bus.if_reg, i);

                    // Disable IME.
                    self.ime = false;

                    // Two wait states are executed every ISR.
                    self.internal_cycle(bus);
                    self.internal_cycle(bus);

                    // Push PC onto the stack.
                    let [lower, upper] = self.reg.pc.to_le_bytes();

                    self.reg.sp = self.reg.sp.wrapping_sub(1);
                    self.write_byte(bus, self.reg.sp, upper);

                    self.reg.sp = self.reg.sp.wrapping_sub(1);
                    self.write_byte(bus, self.reg.sp, lower);

                    // 0x40 - VBLANK
                    // 0x48 - LCD STAT
                    // 0x50 - Timer
                    // 0x58 - Serial
                    // 0x60 - Joypad
                    self.reg.pc = 0x40 + (0x08 * i);
                    self.internal_cycle(bus);

                    // Service only one interrupt at a time.
                    break;
                }
            }
        }
    }

    /// Execute the next opcode, while checking for interrupts.
    /// Return the amount of cycles it took to execute the instruction.
    pub fn execute_next(&mut self, bus: &mut Bus) -> u32 {
        self.cycles = 0;

        // Handle pending interrupts.
        self.handle_interrupts(bus);

        // If the CPU is in HALT state, it just burns one M cycle.
        if self.state == CpuState::Halted {
            self.internal_cycle(bus);
        } else {
            // Fetch the opcode.
            let opcode = self.imm_byte(bus);

            // Decode and execute it.
            self.decode_and_execute(bus, opcode);
        }

        self.cycles >> (self.is_double_speed as u8)
    }
}
