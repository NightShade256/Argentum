# Argentum GB

A simple Game Boy (DMG) emulator written in Rust.

## About

This project is a just a toy emulator to know more about emulation
and have fun while doing so.

This is not a complete emulator by any means, nor is it bug free.
This is not an accurate emulator, nor should it be taken as a reference.

This emulator implements the almost all things except,

1. Audio Processing Unit (APU)
2. MBC1, MBC5 and more...
3. CGB Mode.

## Building

Just run

```bash
cargo build --release
```

and to execute a ROM,

```bash
./argentum-gb <ROM FILE with EXTENSION>
```

## Acknowledgements

The emulator would not be possible without the following resources,

### Documentation and References

1. https://gbdev.io/pandocs/
2. https://izik1.github.io/gbops/index.html
3. https://rgbds.gbdev.io/docs/v0.4.1/gbz80.7
4. https://hacktix.github.io/GBEDG/
5. https://gekkio.fi/files/gb-docs/gbctr.pdf
6. https://cdn.discordapp.com/attachments/465586075830845475/742438340078469150/SM83_decoding.pdf

### Other Emulators

1. BGB and its excellent debugger.
2. https://github.com/Gekkio/mooneye-gb
3. https://github.com/Kappamalone/PurpleBoy
4. https://github.com/wheremyfoodat/Beeg-Boy
5. https://github.com/mohanson/gameboy

### Blogs and Talks

1. http://emudev.de/gameboy-emulator/overview/
2. http://www.codeslinger.co.uk/ (The website is sadly down, but you can use the wayback machine).
3. https://www.youtube.com/watch?v=HyzD8pNlpwI (Ultimate Game Boy Talk - Michael Steil)

and more...

## License

This project is licensed under the terms of the Apache-2.0 license.
