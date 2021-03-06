use argentum_core::bus::*;
use argentum_core::cpu::*;

fn main() {
    let rom = include_bytes!(r"D:\EmuDev\argentum-gb\src\instr_timing.gb");

    let mut cpu = Cpu::new();
    let mut bus = Bus::new(rom);

    cpu.reg.set_af(0x01B0);
    cpu.reg.set_bc(0x0013);
    cpu.reg.set_de(0x00D8);
    cpu.reg.set_hl(0x014D);

    cpu.reg.sp = 0xFFFE;
    cpu.reg.pc = 0x100;

    loop {
        cpu.execute_next(&mut bus);
    }
}
