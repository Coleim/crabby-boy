# Crabby Boy Developer Guides

This is a small collection of learning-oriented guides written while
building [crabby-boy](https://github.com/), a Game Boy emulator, for
learning purposes.

Each guide is written tutorial-style: slow, explicit, with references to
the authoritative [Pan Docs](https://gbdev.io/pandocs/) whenever a detail
is simplified or skipped.

## Available guides

- [PPU — Background Rendering Guide](./ppu-background.md): how the PPU
  turns VRAM tile data into the background layer of a frame, built around
  the real pixel-FIFO/fetcher mechanism.

## The book: Building a Game Boy Emulator in Rust

A full "for dummies" style tutorial, following the project's real commit
history from the very first `Cargo.toml` up to the current state: a
working CPU, bus, timers, interrupts, APU, joypad, a terminal UI, and a
first (minimal) PPU. It assumes general programming knowledge but zero
prior Game Boy knowledge. Start at
[Chapter 0](./book/00-orientation.md) and work through in order.
