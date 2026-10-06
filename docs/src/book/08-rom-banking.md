# 8. ROM Banking

Chapter 2 parsed the cartridge's "cartridge type" byte but never acted on
it. Here (`136bdd1`) is where that finally matters: supporting games
bigger than what fits in the CPU's directly addressable range.

## The problem: 32KB isn't enough

The CPU can address `0x0000`–`0x7FFF` for ROM — that's 32KB. But plenty of
real games (and `cpu_instrs.gb`, which we've been testing against this
whole time) are bigger than that. The trick cartridge manufacturers used:
put a small extra chip — a **Memory Bank Controller (MBC)** — on the
cartridge itself, between the ROM chip and the console. This chip
intercepts writes to certain addresses and uses them not as "store this
byte in ROM" (you can't write to ROM, it's read-only hardware!) but as
*commands*: "from now on, when the CPU reads from `0x4000`–`0x7FFF`, give
it data from a different 16KB chunk of the much larger ROM instead."

So `0x4000`–`0x7FFF` is a **switchable window**: its contents change
depending on which "bank" (16KB chunk) was last selected, while
`0x0000`–`0x3FFF` always shows the fixed first bank.

## Reading through the active bank

```rust
// src/bus/bus.rs
0x4000..=0x7FFF => {
    let offset: u16 = self.bank_number as u16 * 0x4000;
    self.rom[(offset + (addr - 0x4000)) as usize]
}
```

`0x4000` is the size of one bank (16KB, matching the window size above).
So: take the currently selected `bank_number`, multiply by the bank size
to find where that bank starts in the full ROM `Vec<u8>`, then add the
offset *within* the window the CPU is actually reading from. If
`bank_number` is 2 and the CPU reads `0x4100`, this computes
`2 * 0x4000 + (0x4100 - 0x4000) = 0x8100` in the real underlying ROM data.

## Selecting a bank: writes to ROM aren't really writes

```rust
0x2000..=0x3FFF => {
    // Enable bank
    self.bank_number = val & 0b0011111; // keep only the low 5 bits
    if self.bank_number == 0 {
        self.bank_number = 1;
    }
}
```

This is the detail that trips people up the first time: the CPU is
"writing to ROM," which sounds contradictory, but the MBC chip never
actually stores that byte anywhere as data — it just *notices* the write
happened and *reacts* to it by changing internal state (here,
`bank_number`). From the game code's perspective, it looks exactly like
writing to memory; in reality, every such "write" is intercepted and
reinterpreted as a command. This `0x2000..=0x3FFF` match arm in
`Bus::write` is where that interception happens.

The `val & 0b0011111` masks the write down to 5 bits (this early MBC1-style
implementation supports up to 32 banks that way), and the "if it's 0, make
it 1" rule reflects a real MBC quirk: bank 0 is already always visible at
`0x0000`–`0x3FFF`, so selecting "bank 0" for the switchable window would
be redundant — hardware simply treats a request for bank 0 as a request
for bank 1 instead.

## What's still a placeholder here

```rust
0x0000..=0x1FFF => {
    println!("RAM Enable (Write Only)");
}
// ...
0x6000..=0x7FFF => {
    println!("Banking Mode Select (Write Only): {:02x}", val);
}
```

Real MBC1 cartridges have more features than simple ROM bank selection:
enabling/disabling external RAM (`0x0000`–`0x1FFF`) and a banking "mode"
switch affecting how the upper address bits are interpreted
(`0x6000`–`0x7FFF`). At this point they're acknowledged (so the game's
writes don't silently vanish into the generic catch-all) but not acted
upon — logged and left for later refinement, the same bootstrap technique
from Chapter 6 (placeholder first, real behavior once it's actually
needed).

## A small but important `STOP` fix, in passing

```rust
0x10 => {
    self.stopped = true; // STOP n8 2  4
    next_pc = next_pc.wrapping_add(1);
}
```

`STOP` is a 2-byte instruction (opcode + one operand byte, conventionally
always `0x00`), not 1 byte — this commit fixes `next_pc` to actually skip
that operand byte. A good reminder: even deep into later chapters, you'll
occasionally circle back and fix small mistakes from much earlier
chapters, often while working on something unrelated (ROM banking, here).
That's completely normal.

## What we have now

- Working ROM bank switching, letting `cpu_instrs.gb` (and any other ROM
  using this style of banking) load and run correctly beyond 32KB.
- Acknowledged (if not yet implemented) RAM-enable and banking-mode
  writes.
- A corrected `STOP` instruction length.

## What's still missing

- No actual external RAM banking yet (`eram` exists as a flat array in
  `Bus`, with no bank switching of its own).
- Only one MBC "family" of behavior is modeled — real cartridges can use
  several different MBC chip designs (MBC1, MBC3, MBC5, ...) with
  different quirks; this project supports the common simple case needed
  by its test ROMs, not every variant.
- Still no interrupts firing from anywhere, no real timer-driven
  interrupt, no PPU behavior — Part IV picks that up next.
