# Summary

[Introduction](./README.md)

# Guides

- [PPU — Background Rendering Guide](./ppu-background.md)

# Building a Game Boy Emulator in Rust

- [0. What's Actually Inside a Game Boy](./book/00-orientation.md)

- [Part I — First Bytes]()
  - [1. Project Setup & Loading a ROM](./book/01-project-setup.md)
  - [2. Parsing the Cartridge Header](./book/02-cartridge-header.md)

- [Part II — The Brain: Building the CPU]()
  - [3. Registers, Flags, and Your First Opcode](./book/03-cpu-registers-first-opcode.md)
  - [4. Growing the Instruction Set](./book/04-instruction-set.md)
  - [5. The Halt Bug, Part 1](./book/05-halt-bug-part1.md)

- [Part III — Growing Up: Giving the Emulator a Real Architecture]()
  - [6. Splitting into CPU / Bus / Hardware](./book/06-cpu-bus-hardware-split.md)
  - [7. Filling In Missing Registers](./book/07-missing-registers-stubs.md)
  - [8. ROM Banking](./book/08-rom-banking.md)

- [Part IV — Keeping Time]()
  - [9. Timers](./book/09-timers.md)
  - [10. Memory Timing Correctness](./book/10-memory-timing.md)
  - [11. Test Infrastructure & CI](./book/11-testing-and-ci.md)

- [Part V — A First PPU Stub]()
  - [12. Scanlines 101 & a Minimal PPU](./book/12-minimal-ppu.md)
  - [13. The Halt Bug, Part 2](./book/13-halt-bug-part2.md)
  - [14. Interrupts Done Properly](./book/14-interrupts-done-properly.md)

- [Part VI — Sound: The APU]()
  - [15. APU Architecture & the audio/ Module](./book/15-apu-architecture.md)
  - [16. Channel 1 — Square Wave with Sweep](./book/16-apu-channel1.md)
  - [17. Channel 2 — The Simpler Square Wave](./book/17-apu-channel2.md)
  - [18. Channel 3 — Custom Wave](./book/18-apu-channel3.md)
  - [19. Channel 4 — Noise](./book/19-apu-channel4.md)
  - [20. Passing Blargg's dmg_sound Tests](./book/20-apu-passing-tests.md)

- [Part VII — Input]()
  - [21. The Joypad](./book/21-joypad.md)

- [Part VIII — Giving It a Face: Building a Terminal UI]()
  - [22. A Detour That Didn't Stick](./book/22-detour-image-test.md)
  - [23. Decoupling Emulation Speed from Display Refresh](./book/23-decoupling-tick-from-display.md)
  - [24. A High-Pass Filter for Cleaner Audio](./book/24-audio-high-pass-filter.md)
  - [25. Displaying CPU Registers and Cartridge Info](./book/25-terminal-ui-registers-header.md)
  - [26. Visualizing VRAM](./book/26-vram-tile-viewer.md)

- [Part IX — Back to the PPU]()
  - [27. Where We Left the PPU, and What's Next](./book/27-back-to-the-ppu.md)
