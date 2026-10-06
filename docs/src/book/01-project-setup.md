# 1. Project Setup & Loading a ROM

Every emulator project starts the same humble way: a Rust project that
can read a game file into memory and print something about it. This
chapter matches the very first commit of crabby-boy.

## The Cargo project

```toml
# Cargo.toml
[package]
name = "CrabbyBoy"
version = "0.1.0"
edition = "2024"

[dependencies]
```

Nothing special yet — a plain binary crate, no dependencies. Emulators are
a great excuse to write a lot of code with very few external libraries,
since most of the work is "faithfully compute what the hardware would
compute," not "glue existing libraries together."

## What is a ROM file, physically?

A Game Boy cartridge is, from the CPU's point of view, just more
addressable memory — a chunk of bytes the CPU can read instructions and
data from. A `.gb` ROM file is a byte-for-byte dump of that cartridge's
ROM chip. So "loading a ROM" is as simple as reading the file into a
`Vec<u8>`:

```rust
// src/main.rs
mod memory;
use memory::Memory;

fn main() {
    let file_path = "./cpu_instrs.gb";
    let rom = std::fs::read(file_path).unwrap();
    let memory = Memory::new(&rom);

    let opcode = rom[0];
    println!("opcode: 0b{:08b} - 0x{:X}", opcode, opcode);

    let logo: [u8; 48] = read_nintendo_logo(&rom);
    print_logo_ascii(&logo);
}

fn read_nintendo_logo(rom: &[u8]) -> [u8; 48] {
    let logo_start = 0x0104;
    let logo_end = 0x0133;
    let mut logo: [u8; 48] = [0; 48];
    logo.copy_from_slice(&rom[logo_start..=logo_end]);
    logo
}

fn print_logo_ascii(logo: &[u8; 48]) {
    for str in logo {
        print!("{:02X} ", str);
    }
}
```

A few things worth noticing, since they set the tone for the whole
project:

- `rom[0]` — the very first byte of the file is already meaningful: it's
  the first CPU instruction the console will execute. We print it in both
  binary and hex because you'll constantly be translating between the two
  when reading hardware documentation (hex is compact; binary makes bit
  patterns/flags obvious).
- `0x0104..=0x0133` is a fixed region inside the ROM reserved for the
  **Nintendo logo** bitmap — the real hardware's boot ROM refuses to start
  a game whose logo bytes don't match exactly (this is how Nintendo
  enforced licensing). We'll do our own validation of this later
  (Chapter 2) — for now we just read and print it.

## A first (very small) model of memory

```rust
// src/memory.rs
pub struct Memory {
    data: [u8; 0x10000],
}

impl Memory {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = Memory { data: [0; 0x10000] };
        memory.data[0x0000..(0x0000 + rom.len())].copy_from_slice(rom);
        memory
    }
}
```

`0x10000` is 65,536 in decimal — the entire address space a 16-bit address
can reach, as explained in Chapter 0. This first version is intentionally
naive: it just copies the whole ROM to the start of one big flat array,
with no understanding yet of "this part of the address space means VRAM,"
or "ROMs bigger than 32KB need bank switching." That's fine — this file
gets completely rewritten more than once as the project grows (see
Chapter 6's `Bus` and Chapter 8's ROM banking). The point of a first
commit is to have *something* that compiles and does one honest, useful
thing.

## What we have now

- A Rust project that reads a `.gb` file into memory.
- A flat 64KB memory array.
- Printing the first opcode byte and the Nintendo logo bytes, as a sanity
  check that the file is being read correctly.

## What's still missing

- No actual CPU yet — nothing executes any instructions.
- No real understanding of what's inside a cartridge header (next
  chapter).
- No separation between ROM, RAM, VRAM, I/O registers, etc. — it's all one
  array for now.
