# 4. Growing the Instruction Set

This chapter covers the longest, most repetitive stretch of the whole
project: turning a handful of opcodes into (almost) all 512 of them (256
"main" opcodes, plus 256 more behind a special `0xCB` prefix). It spans a
long sequence of commits (`ab86d28` → `dc9a258`). Instead of walking every
single opcode (that's what the
[opcode tables](https://gbdev.io/gb-opcodes/optables/) are for), this
chapter focuses on the handful of *ideas* that, once understood, make
every individual opcode straightforward.

## A first mid-grind reorganization: `Bus` is born

Partway through adding opcodes, instructions started needing real memory
regions (VRAM, WRAM, OAM, HRAM) instead of one flat array, so a `Bus`
struct appears (`a6f2e4e`), already annotated with the full memory map
from Chapter 0:

```rust
// src/cpu/bus.rs
pub struct Bus {
    rom: Vec<u8>,
    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0x2000],
    serial: Serial,
    hram: [u8; 0x2000],
    ie: u8,
}

impl Bus {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => self.rom[addr as usize], // no banking yet
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.ie,
            _ => { println!("[BUS] Not mapped addressed {}", addr); 0 }
        }
    }
    // ...
}
```

Rust's `match` on a range (`0x8000..=0x9FFF => ...`) is doing exactly the
job of the memory map table from Chapter 0 — each arm is one row of that
table. From here on, `CPU::execute` takes a `&mut Bus` instead of a raw
`&mut [u8]`, and every memory access goes through `bus.read`/`bus.write`
instead of indexing an array directly. This one change is what makes
later chapters possible (ROM banking, I/O registers, VRAM, OAM all need
*different* behavior per address range, not just a different backing
array).

## Idea 1: register pairs are just two registers glued together

Several instructions treat `B`+`C`, `D`+`E`, or `H`+`L` as one 16-bit
value (e.g. `LD BC, nn`, `INC HL`). There's no new hardware concept here —
it's the exact same little-endian combination trick from `read16bytes` in
Chapter 3, just exposed as convenient getter/setter pairs:

```rust
fn get_bc(&self) -> u16 {
    (self.b as u16) << 8 | self.c as u16
}
fn set_bc(&mut self, value: u16) {
    self.b = (value >> 8) as u8;
    self.c = value as u8; // truncates to the low byte
}
```

## Idea 2: flags, accessed through named helpers instead of raw bit math

Writing `self.f |= 0x80` everywhere (Chapter 3) gets error-prone fast.
By this point in the grind, flag access becomes small helper methods —
`get_z`/`set_z`, `get_n`/`set_n`, `get_h`/`set_h`, `get_c`/`set_c` — so
instruction bodies read like the spec, not like bit-twiddling:

```rust
self.set_h(false);
self.set_z(self.a == 0);
```

If you implement your own CPU, write these helpers *before* you need
them in the 20th instruction — it pays for itself almost immediately.

## Idea 3: the `0xCB` prefix is a second opcode table

If the byte at `PC` is `0xCB`, that byte doesn't mean an instruction by
itself — it means "the *next* byte selects from a completely separate
table of 256 bit-manipulation instructions" (rotates, shifts, and
per-bit test/set/clear operations). Rather than writing 256 more
`match` arms by hand, the actual CB-prefixed byte gets decoded by
splitting it into 3 bit-fields, because the people who designed this CPU
deliberately laid the opcode byte out that way:

```rust
let category: u8 = opcode >> 6;              // top 2 bits:   which family (rotate/shift, BIT, RES, SET)
let subcategory: u8 = opcode >> 3 & 0b0000_0111; // middle 3 bits: which operation, or which bit number for BIT/RES/SET
let operand: u8 = opcode & 0b0000_0111;         // bottom 3 bits: which register (or (HL))

match category {
    0 => match subcategory {
        0 => self.rlc(operand, bus),
        1 => self.rrc(operand, bus),
        2 => self.rl(operand, bus),
        3 => self.rr(operand, bus),
        4 => self.sla(operand, bus),
        5 => self.sra(operand, bus),
        6 => self.swap(operand, bus),
        7 => self.srl(operand, bus),
        _ => {}
    },
    1 => self.bit(subcategory, operand, bus), // BIT b, r
    2 => { /* RES b, r */ }
    3 => { /* SET b, r */ }
    _ => {}
}
```

This is a nice general lesson, not just a Game Boy one: when a spec's
binary layout looks suspiciously tidy (exactly 2 + 3 + 3 bits, lining up
with "8 operations × 8 registers," or "4 categories × 8 bit-numbers × 8
registers"), it's almost always intentional, and decoding by shifting
and masking beats writing out all 256 cases by hand.

## Idea 4: `DAA` — decimal adjust, a Game Boy oddity worth seeing once

Most instructions are intuitive once you know the CPU concepts. `DAA`
("Decimal Adjust Accumulator") is the one instruction in the whole set
that looks like black magic the first time you read it:

```rust
0x27 => {
    // DAA — see https://rgbds.gbdev.io/docs/v1.0.0/gbz80.7#DAA
    let mut adjustment = 0;
    if self.get_n() {
        if self.get_h() { adjustment += 0x06; }
        if self.get_c() { adjustment += 0x60; }
        self.a = self.a.wrapping_sub(adjustment);
    } else {
        if self.get_h() || (self.a & 0xF) > 0x9 { adjustment += 0x06; }
        if self.get_c() || self.a > 0x99 {
            adjustment += 0x60;
            self.set_c(true);
        }
        self.a = self.a.wrapping_add(adjustment);
    }
    self.set_h(false);
    self.set_z(self.a == 0);
}
```

Context that makes this make sense: `DAA` exists to make binary addition
*behave like* decimal addition, for programs that store numbers as
"packed BCD" (binary-coded decimal — each nibble of a byte represents one
decimal digit, 0-9, instead of the byte representing one combined binary
number 0-255). After an `ADD`/`SUB` on BCD-encoded values, `DAA` patches up
the result so each nibble is back in the valid 0-9 range, using the `H`
and `C` flags set by the *previous* instruction to know whether a nibble
or byte "carried over." You will almost certainly never need to
understand this more deeply than "copy the algorithm correctly, test it
against Blargg's test ROM, move on" — which is a perfectly fine approach
for details this far into hardware arcana.

## Idea 5: you will under-implement opcodes first, and that's fine

Throughout this grind, the `_ => {}` catch-all arm (seen since Chapter 3)
stays in place, and individual opcodes get commented out and reinstated
as bugs are found:

```rust
// 0x38 => {
//     println!("SRL B 2  8 Z 0 0 C")
// }
_ => {
    println!("Unimplemented opcode: 0xCB{:02X} at PC: 0x{:04X}", opcode, current_pc);
}
```

This is a completely normal way to build a CPU core: implement an
instruction, test it, find it was subtly wrong (wrong flag, wrong cycle
count, forgot to advance `PC`), comment it out while you fix the
underlying helper function, then re-enable it. Don't aim for a perfect
single pass over all 256+256 opcodes — aim for a loop of
implement → test → fix.

## How do you know if you got it right? Blargg's `cpu_instrs.gb`

This whole chapter's commits are validated the same way: by running a
well-known community test ROM,
[Blargg's `cpu_instrs.gb`](https://github.com/retrio/gb-test-roms)
(already present in `tests/cpu_instrs/`, split into 11 sub-tests: 
`01-special.gb`, `02-interrupts.gb`, and so on). This ROM exercises CPU
instructions and reports **PASS** or **FAIL** as text, by writing
characters out over the Game Boy's **serial port** — a simple
communication link that, on real hardware, would talk to a cable
connecting two consoles, and which test ROM authors long ago repurposed
as a convenient way to print debug text out of a running ROM with no
screen required. We capture that serial output and print it, long before
we have any screen to display a real PASS/FAIL message on. Chapter 11
turns this into a proper automated test suite.

## What we have now

- A `Bus` struct, routing reads to the right memory region.
- Register pairs (`BC`/`DE`/`HL`) and flag helper methods.
- Almost the entire main opcode table, plus the `0xCB`-prefixed table via
  bit-field decoding.
- A way to validate correctness against a real community test ROM via
  serial output.

## What's still missing

- `0xCB` `RES`/`SET` categories are stubbed (see the empty `2 => {}` /
  `3 => {}` arms above) at this exact point in history — filled in soon
  after.
- `HALT` exists as an opcode by name but its famous hardware bug isn't
  handled yet — that's Chapter 5, immediately next.
- No interrupts yet, even though `02-interrupts.gb` is already sitting in
  the test folder, waiting.
- Still no I/O registers, timer, or PPU — the CPU can compute, but there's
  nothing yet for it to meaningfully control.
