use argentum_core::bus::*;
use argentum_core::cpu::*;

fn main() {
    let rom = include_bytes!("instr_timing.gb");
    let mut bus = Bus::new(rom);
    let mut cpu = Cpu::new();

    cpu.skip_bootrom();

    loop {
        let cycles = cpu.execute_opcode(&mut bus);
        bus.tick_components(cycles);
    }
}
