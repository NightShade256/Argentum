//! Contains an implementation of the Sharp SM83 CPU
//! found inside the Game Boy.

use std::fmt::*;

mod decode;
mod instructions;
mod registers;

use self::registers::*;
use crate::bus::Bus;

// The timing information found below is taken from the emulator
// Purple Boy written by Kappamalone, [https://github.com/Kappamalone/PurpleBoy]
// with their permission.

/// M-cycles taken by unprefixed opcodes to run.
const UNPREFIXED_TIMINGS: [u32; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, 1, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1,
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, 2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, 2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4,
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, 3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4,
];

/// M-cycles taken by CB prefixed opcodes to run.
const PREFIXED_TIMINGS: [u32; 256] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2,
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
];

/// Implementation of the Sharp SM83.
pub struct Cpu {
    /// Set of all the registers.
    pub r: Registers,

    /// Interrupt Master Enable switch.
    pub ime: bool,

    /// Is the CPU currently halted or running?
    pub halted: bool,

    /// The cycles it took to execute ONLY the current instrucion.
    pub cycles: u32,
}

/// Formatted similar to wheremyfoodat's emulation logs.
impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let reg_one = format!(
            "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X}",
            self.r.a,
            self.r.f.bits(),
            self.r.b,
            self.r.c,
            self.r.d
        );

        let reg_two = format!(
            "E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X}",
            self.r.e, self.r.h, self.r.l, self.r.sp, self.r.pc,
        );

        write!(f, "{} {}", reg_one, reg_two)
    }
}

impl Cpu {
    /// Create a new `Cpu` instance.
    pub fn new() -> Self {
        Self {
            r: Registers::new(),
            ime: false,
            halted: false,
            cycles: 0,
        }
    }

    /// Skips the bootrom, and initializes default values for
    /// registers.
    pub fn skip_bootrom(&mut self) {
        self.r.write_r16(Reg16::AF, 0x01B0);
        self.r.write_r16(Reg16::BC, 0x0013);
        self.r.write_r16(Reg16::DE, 0x00D8);
        self.r.write_r16(Reg16::HL, 0x014D);

        self.r.sp = 0xFFFE;
        self.r.pc = 0x0100;
    }

    /// Read a byte from the address pointed to by `PC`.
    pub fn imm_byte(&mut self, bus: &mut Bus) -> u8 {
        let value = bus.read_byte(self.r.pc);
        self.r.pc += 1;

        value
    }

    /// Read a word from the address pointed to by `PC`.
    pub fn imm_word(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.r.pc);
        self.r.pc += 2;

        value
    }

    /// Handle pending interrupts.
    fn handle_interrupts(&mut self, bus: &mut Bus) {
        let interrupts = bus.ie_flag & bus.if_flag;

        // If there are pending interrupts, CPU should be
        // back up and running.
        if interrupts != 0 {
            self.halted = false;
        }

        // If IME is not enabled, we don't service the interrupt.
        if !self.ime {
            return;
        }

        if interrupts != 0 {
            for i in 0..5 {
                if (bus.ie_flag & (1 << i) != 0) && (bus.if_flag & (1 << i) != 0) {
                    // Disable the interrupt in IF.
                    bus.if_flag &= !(1 << i);

                    // Disable IME.
                    self.ime = false;

                    // Jump to the the service address.
                    self.push(bus, self.r.pc);

                    // 0x40 - VBLANK
                    // 0x48 - LCD STAT
                    // 0x50 - Timer
                    // 0x58 - Serial
                    // 0x60 - Joypad
                    self.r.pc = 0x40 + (0x08 * i);

                    // As in Pan Docs, ISR takes 5 M cycles.
                    self.cycles += 20;

                    // Service only one interrupt at a time.
                    break;
                }
            }
        }
    }

    /// Execute one opcode and return the amount of T-cycles
    /// it took to run it.
    pub fn execute_opcode(&mut self, bus: &mut Bus) -> u32 {
        self.cycles = 0;

        self.handle_interrupts(bus);

        if self.halted {
            self.cycles += 4;
        } else {
            // Fetch opcode.
            let opcode = self.imm_byte(bus);

            // Add correct amount of cycles.
            self.cycles += if opcode == 0xCB {
                PREFIXED_TIMINGS[bus.read_byte(self.r.pc) as usize]
            } else {
                UNPREFIXED_TIMINGS[opcode as usize]
            } * 4;

            // Decode and execute the opcode.
            self.decode_and_execute(bus, opcode);
        }

        self.cycles
    }
}
