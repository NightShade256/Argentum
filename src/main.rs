use argentum_core::bus::*;
use argentum_core::cpu::*;

use std::io::prelude::*;

fn main() {
    let rom = include_bytes!(r"D:\EmuDev\argentum-gb\src\instr_timing.gb");

    let mut cpu = CPU::new();
    let mut bus = Bus::new(rom);

    cpu.reg.set_af(0x01B0);
    cpu.reg.set_bc(0x0013);
    cpu.reg.set_de(0x00D8);
    cpu.reg.set_hl(0x014D);

    cpu.reg.sp = 0xFFFE;
    cpu.reg.pc = 0x0100;

    //let mut file = std::fs::File::create("./logs.txt").unwrap();

    loop {
        // writeln!(
        //     file,
        //     "{} ({:02X} {:02X} {:02X} {:02X})",
        //     &cpu,
        //     bus.read_byte(cpu.reg.pc),
        //     bus.read_byte(cpu.reg.pc + 1),
        //     bus.read_byte(cpu.reg.pc + 2),
        //     bus.read_byte(cpu.reg.pc + 3)
        // )
        // .unwrap();
        cpu.handle_interrupts(&mut bus);

        if cpu.state == CpuState::Halted {
            cpu.internal_cycle(&mut bus);
        } else {
            let opcode = cpu.imm_byte(&mut bus);
            cpu.decode_and_execute(&mut bus, opcode);
        }
    }
}
