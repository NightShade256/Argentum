use argentum_core::GameBoy;

fn main() {
    let rom = include_bytes!(r"D:\EmuDev\argentum-gb\src\instr_timing.gb");

    let mut gb = GameBoy::new(rom);

    loop {
        gb.execute_frame();
    }
}
