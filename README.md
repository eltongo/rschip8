# rschip8

This is a chip8 emulator written in Rust. I'm in the process of learning Rust, so I thought what better way to test my skills than writing a good old-fashioned emulator?

## How to build?

Building should be straightforward, except for SDL2, which rschip8 depends on. But that shouldn't be too difficult. You can find installation instructions for different platforms [here](https://github.com/Rust-SDL2/rust-sdl2). Also, [here's a link](https://github.com/Rust-SDL2/rust-sdl2#homebrew) to the specific section that shows the instructions I followed for macOS using Homebrew.

## What about usage?

Usage is very simple. After building the target using `cargo build --release`, the path of the created binary will be `./target/release/rschip8`. Usage is as follows:

```
./rschip8 /path/to/chip8.rom
```

Alternatively, you can use `cargo run` to compile and run simultaneously:

```
cargo run -- /path/to/chip8.rom
```
