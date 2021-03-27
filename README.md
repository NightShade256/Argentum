# Argentum GB

A Game Boy emulator written in Rust.

## About

Argentum GB is a Game Boy emulator I wrote during 2021.
It is a fairly accurate emulator that tries to achieve sub-instruction (M-cycle) accuracy.

## Features

1. Support for MBC[1/3/5] and ROM only cartridges.
2. Accurate CPU core which passes Blargg's CPU instruction,
   instruction timing and memory timing tests.
3. Scanline based PPU which passes the dmg-acid2 test.
4. Full featured audio emulation.

## Building

You can build the project using `cargo`.

```bash
cargo build --release
```

and to execute a ROM,

```bash
./argentum-gb <ROM FILE>
```

Binaries are provided in the GitHub Releases section.

## Screenshots

### Games

<img src="./assets/Pokemon.png" width="300"> &nbsp;
<img src="./assets/Mario.png" width="300"> &nbsp;
<img src="./assets/Tetris.png" width="300"> &nbsp;

### Test ROM(s)

<img src="./assets/cpu_instrs.png" width="300"> &nbsp;
<img src="./assets/dmg_acid2.png" width="300"> &nbsp;

## Acknowledgements

The emulator would not be possible without the following resources,

### Documentation and References

1. [Pandocs](https://gbdev.io/pandocs/)
2. [Izik's Opcode Map](https://izik1.github.io/gbops/index.html)
3. [Optix's GBEDG](https://hacktix.github.io/GBEDG/)
4. [Game Boy - Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
5. [wheremyfoodat's SM83 Instruction Decoding Guide](https://cdn.discordapp.com/attachments/465586075830845475/742438340078469150/SM83_decoding.pdf)

### Other Emulators

1. [BGB](http://bgb.bircd.org/)
2. [Mooneye GB - Gekkio](https://github.com/Gekkio/mooneye-gb)
3. [Purple Boy - Kappamalone](https://github.com/Kappamalone/PurpleBoy)
4. [CryBoy - Matthew Berry](https://github.com/mattrberry/CryBoy)

### Blogs and Talks

1. [[emudev]](http://emudev.de/gameboy-emulator/overview/)
2. [Ultimate Game Boy Talk - Michael Steil](https://www.youtube.com/watch?v=HyzD8pNlpwI)

## License

This project is licensed under the terms of the Apache-2.0 license.
