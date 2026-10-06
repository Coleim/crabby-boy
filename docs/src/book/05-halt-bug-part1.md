# 5. The Halt Bug, Part 1

Tucked into the tail end of the opcode grind (`dc9a258`, with a dedicated
test ROM added right after in `5631832`) is one of the Game Boy CPU's
most infamous quirks: the **HALT bug**. It's a great first example of
something this book will come back to more than once — real hardware
sometimes behaves in ways that look like obvious bugs, except they're not
bugs, they're exactly what the chip does, and your emulator has to
reproduce the "bug" on purpose.

## What `HALT` is supposed to do

`HALT` (opcode `0x76`) tells the CPU "stop executing instructions and do
nothing until an interrupt happens." This saves power on real hardware,
and in software terms it's simply: stop advancing, keep ticking other
components (timer, PPU, APU), and resume once an interrupt becomes
pending.

```rust
pub halt: bool,
```

A single boolean flag. When true, the main loop skips CPU execution
entirely (we'll see this loop in Chapter 11) until something sets it back
to false.

## The quirk: `HALT` with interrupts pending but disabled

Here's the oddity. There are two independent concepts:

- **IME** ("Interrupt Master Enable") — a CPU-internal switch: are
  interrupts allowed to actually interrupt the CPU right now at all?
- **IE** (`0xFFFF`) and **IF** (`0xFF0F`) — which interrupt *types* are
  enabled, and which are currently pending, regardless of IME.

If a game executes `HALT` at the exact moment `IME` is off, but some
interrupt is both enabled (`IE`) and already pending (`IF`), real
hardware does **not** halt at all. Instead, it immediately continues
execution — but with a bug: **the next instruction's opcode byte gets
fetched twice**, i.e. `PC` fails to advance for one fetch, silently
re-executing whatever single-byte effect the next opcode had. This is
exactly what the name "HALT bug" refers to — not a bug in an emulator,
but a documented quirk of the real chip that every accurate emulator must
reproduce.

## First attempt at implementing it

```rust
pub halt_bug: bool,
```

```rust
0x76 => {
    let ie = bus.get_ie();
    let if_flag = bus.get_io().get_if();

    if !self.ime && (ie & if_flag) != 0 {
        // HALT BUG — don't halt, just corrupt next fetch
        self.halt_bug = true;
    } else {
        self.halt = true;
    }
}
```

And then, at the very top of the fetch step:

```rust
let mut next_pc: u16 = if self.halt_bug {
    self.halt_bug = false;
    self.pc // don't advance — re-fetch the same address next time
} else {
    self.pc.wrapping_add(1)
};
```

The logic in plain language: when `HALT` runs, check whether `IME` is off
*and* an enabled, pending interrupt already exists. If so, set
`halt_bug = true` instead of `halt = true`. Next time an opcode is
fetched, if `halt_bug` was set, don't advance `PC` past the opcode we're
about to execute — meaning the *following* fetch will read that same byte
location again. That's the "duplicate fetch" effect, reproduced.

## Why this gets its own test ROM

Edge-case CPU behavior like this is exactly the kind of thing that's easy
to implement subtly wrong (off-by-one in exactly *which* byte gets
re-fetched, or getting the IME/IE/IF condition slightly wrong), and
subtly wrong versions can still pass most games by luck while failing
specific, deliberately crafted test cases. That's why the community
maintains targeted test ROMs like `tests/halt_bug.gb`, added here
specifically to validate this one behavior in isolation, independent of
the broader `cpu_instrs.gb` suite from Chapter 4.

## What we have now

- A `halt` flag that stops CPU execution until an interrupt.
- A first attempt at the HALT bug: detecting the "IME off, interrupt
  already pending" condition and corrupting the next fetch instead of
  truly halting.
- A dedicated test ROM to validate this behavior.

## What's still missing

- This first attempt is **not fully correct yet** — notice the condition
  `!self.ime && (ie & if_flag) != 0` is checked, but there's still a gap
  in exactly how the corrupted fetch interacts with multi-byte
  instructions, which surfaces as a real bug later. Chapter 13 ("The Halt
  Bug, Part 2") comes back to fix it properly, once there's an actual
  interrupt source (the PPU's VBlank, from Part V) to trigger it against
  in practice rather than only in the isolated test ROM.
- No interrupt *handling* exists yet at all at this point (no jumping to
  interrupt vectors, no `IE`/`IF` clearing on dispatch) — just enough of
  the `IME`/`IE`/`IF` concept to make this one instruction's quirk
  testable. Full interrupt handling is Chapter 14.
