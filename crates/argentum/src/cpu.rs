mod decode;
mod instructions;
mod registers;

use self::registers::Registers;
use crate::{bus::Bus, helpers::BitExt};

/// Enumerates all the states the CPU can be in.
#[derive(PartialEq)]
pub enum CpuState {
    Halted,
    Running,
}

/// Implementation of the Sharp SM83 CPU.
pub struct Cpu {
    /// The amount of T-cycles taken by the current instruction.
    cycles: u32,

    /// The Interrupt Master Enable flag.
    ime: bool,

    /// Indicates whether we are emulating the CGB or the DMG.
    is_cgb: bool,

    /// Registers associated with the CPU.
    reg: Registers,

    /// The current CPU state.
    state: CpuState,
}

impl Cpu {
    /// Create a new `CPU` instance.
    pub fn new(is_cgb: bool) -> Self {
        Self {
            cycles: 0,
            ime: false,
            is_cgb,
            reg: Registers::new(),
            state: CpuState::Running,
        }
    }

    /// Initalize the CPU to post-bootrom state.
    pub fn skip_bootrom(&mut self, is_cgb: bool) {
        if is_cgb {
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
        bus.tick_components(4);
    }

    /// Read a byte from the address contained within the PC, and increment
    /// PC afterwards.
    pub fn imm_byte(&mut self, bus: &mut Bus) -> u8 {
        self.reg.pc = self.reg.pc.wrapping_add(1);
        self.read_byte(bus, self.reg.pc.wrapping_sub(1))
    }

    /// Handle pending interrupts (if any) one at a time.
    pub fn handle_interrupts(&mut self, bus: &mut Bus) {
        let if_reg = bus.get_if();
        let ie_reg = bus.get_ie();

        let interrupts = *ie_reg & *if_reg;

        if self.state == CpuState::Halted && interrupts != 0 {
            self.state = CpuState::Running;
        }

        if !self.ime {
            return;
        }

        if interrupts != 0 {
            for i in 0..5 {
                if ie_reg.bit(i as u32) && if_reg.bit(i as u32) {
                    bus.get_if_mut().res(i as u32);
                    self.ime = false;

                    self.internal_cycle(bus);
                    self.internal_cycle(bus);

                    let [lower, upper] = self.reg.pc.to_le_bytes();

                    self.reg.sp = self.reg.sp.wrapping_sub(1);
                    self.write_byte(bus, self.reg.sp, upper);

                    self.reg.sp = self.reg.sp.wrapping_sub(1);
                    self.write_byte(bus, self.reg.sp, lower);

                    self.reg.pc = 0x40 + (0x08 * i);
                    self.internal_cycle(bus);

                    return;
                }
            }
        }
    }

    /// Execute the next instruction, while checking for interrupts and
    /// return the amount of cycles it took to execute the instruction.
    pub fn execute_next(&mut self, bus: &mut Bus) -> u32 {
        self.cycles = 0;
        self.handle_interrupts(bus);

        if self.state == CpuState::Halted {
            self.internal_cycle(bus);
        } else {
            let instruction = self.imm_byte(bus);
            self.decode_and_execute(bus, instruction);
        }

        self.cycles
    }
}
