# 3. Registers, Flags, and Your First Opcode

Time to build the part everyone thinks of first when they hear
"emulator": the CPU.

## The register file

```rust
// src/cpu.rs
pub struct CPU {
    pub a: u8,
    pub f: u8, // F = Z,N,H,C
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}
```

Seven 8-bit general-purpose registers (`A B C D E H L`), one 8-bit flags
register (`F`), and two 16-bit special registers: `PC` (program counter —
always the address of the next instruction to fetch) and `SP` (stack
pointer — address of the top of the call stack, used by `CALL`/`RET`/
`PUSH`/`POP`, which we'll meet properly in Chapter 4).

Many instructions treat two 8-bit registers as one 16-bit pair (`BC`, `DE`,
`HL`, and `AF`) — we don't see that yet in this first commit, but it's
coming almost immediately in Chapter 4.

## Why these specific starting values?

```rust
pub fn new() -> Self {
    CPU {
        a: 0x01,
        f: 0xB0,
        b: 0x00,
        c: 0x13,
        d: 0x00,
        e: 0xD8,
        h: 0x01,
        l: 0x4D,
        pc: 0x0100,
        sp: 0xFFFE,
    }
}
```

These aren't arbitrary. On real hardware, a small internal **boot ROM**
runs first (scrolling the Nintendo logo, playing the startup chime,
validating the header), and by the time it hands control over to the
cartridge at address `0x0100`, it has left the registers in this exact
state. Since we're not emulating the boot ROM itself (yet — some
emulators do, we don't here), we just start the CPU as if the boot ROM
had already run: `PC = 0x0100` (the cartridge's actual entry point,
matching the `entry_point` header field from Chapter 2) and these
specific register values, which are documented, known constants.

## The flags register: 4 bits that matter, packed into one byte

`F` isn't a general-purpose register — each of its top 4 bits is an
independent true/false flag, set or cleared as a *side effect* of running
certain instructions, and later read back by conditional jumps
("jump only if the last result was zero," etc.):

| Bit | Name | Meaning |
|---|---|---|
| 7 | Z | Zero — the last result was 0 |
| 6 | N | Subtract — the last operation was a subtraction |
| 5 | H | Half-carry — a carry occurred out of bit 3 |
| 4 | C | Carry — a carry (or borrow) occurred out of bit 7 |

The bottom 4 bits of `F` are always 0 on real hardware. We'll see exactly
this in the first real instruction below.

## The fetch-decode-execute loop, for real this time

```rust
fn read16bytes(&mut self, mem: &[u8], pc: u16) -> u16 {
    let low = mem[(pc) as usize] as u16;
    let high = mem[(pc + 1) as usize] as u16;
    (high << 8) | low
}

pub fn execute(&mut self, mem: &mut [u8]) -> bool {
    let opcode = mem[self.pc as usize];
    println!("Parsing OP CODE: {:#X}", opcode);
    let mut next_pc: u16 = self.pc + 1;
    match opcode {
        0x00 => {
            println!("NOOP")
        }
        // ... more opcodes below
        _ => {
            println!("Something else");
            return false;
        }
    }
    self.pc = next_pc;
    return true;
}
```

This is the shape every instruction handler will follow from now on:

1. Read the byte at `PC` — that's the **opcode** (operation code), a
   number that identifies which instruction to run.
2. `match` on it to find the right handler.
3. The handler reads however many extra bytes it needs (0, 1, or 2, right
   after the opcode byte) and does its job.
4. Advance `PC` past the opcode and its extra bytes (unless the
   instruction itself changed `PC`, like a jump).

One Game Boy-specific detail worth calling out: `read16bytes` reads two
bytes and combines them as `(high << 8) | low` — the *first* byte in
memory is the **low** byte, the second is the **high** byte. This is
called **little-endian** byte order, and the Game Boy uses it everywhere
multi-byte values appear in memory. If you ever get a 16-bit value
exactly backwards (e.g. reading `0x1234` as `0x3412`), this is almost
always why.

## Your first few real instructions

```rust
0xC3 => {
    // JP nn — jump to a fixed 16-bit address
    let addr = self.read16bytes(mem, next_pc);
    next_pc = addr;
}
0x31 => {
    // LD SP, n16 — load a 16-bit immediate value into SP
    self.sp = self.read16bytes(mem, next_pc);
    next_pc += 2;
}
0x3E => {
    // LD A, n8 — load an 8-bit immediate value into A
    let val: u8 = mem[next_pc as usize];
    self.a = val;
    next_pc += 1;
}
```

Every instruction's comment in the original source (e.g. `"LD SP, n16 3
12"`) is shorthand straight from the opcode reference tables everyone in
the Game Boy dev community uses, e.g.
<https://gbdev.io/gb-opcodes/optables/>: instruction mnemonic, byte
length, and cycle count. Get comfortable with that table — Chapter 4
leans on it constantly.

## `SUB A, n8`: your first flags

```rust
0xD6 => {
    // SUB A, n8 — A = A - n8, 2 bytes, 8 cycles, affects Z N H C
    let val: u8 = mem[next_pc as usize];
    let a = self.a;
    self.a = a.wrapping_sub(val);
    self.f = 0;
    if self.a == 0 {
        self.f |= 0x80; // Z
    }
    self.f |= 0x40; // N always set after a subtraction

    // H: did a borrow happen out of bit 4? (i.e. the low nibble
    // couldn't cover the subtraction on its own)
    if (a & 0xF) < (val & 0xF) {
        self.f |= 0x20;
    }

    // C: did a borrow happen out of bit 8? (i.e. the whole byte
    // couldn't cover the subtraction, result wrapped around)
    if a < val {
        self.f |= 0x10;
    }
    next_pc += 1;
}
```

Two Rust details worth a beginner's pause:

- `wrapping_sub`: plain `a - val` in Rust **panics** (in debug builds) if
  the subtraction would go below 0 for a `u8`. But Game Boy arithmetic is
  expected to wrap around (`0x00 - 0x01 = 0xFF`), exactly like the real
  8-bit hardware would. `wrapping_sub`/`wrapping_add` are how you tell
  Rust "yes, I know, wrap around instead of panicking." You'll use these
  constantly.
- `self.f |= 0x80` (OR-ing in a bit) and `self.a & 0xF` (ANDing to isolate
  the low nibble) are the two bitwise operations you'll use the most in
  this entire project. If bitwise AND/OR/shift are fuzzy for you, it's
  worth a short detour before Chapter 4 — nearly every instruction uses
  them.

## What we have now

- A `CPU` struct with the real register layout and real boot-time values.
- A working fetch-decode-execute loop.
- A handful of instructions: `NOP`, `JP nn`, `LD SP,n16`, `LD [a16],A`,
  `DI` (not yet implemented, just acknowledged), `LD A,n8`, `SUB A,n8`.

## What's still missing

- Only a handful of the 256 possible opcodes exist — everything else
  falls into the `_ => { return false }` catch-all. Chapter 4 fills in
  (almost) the rest.
- No 16-bit register pairs (`BC`/`DE`/`HL`/`AF`) yet, even though many
  instructions need them.
- No connection to the cartridge header or memory bus yet — `execute`
  takes a raw `&mut [u8]` slice directly.
- No interrupts, no cycle-accurate timing return value yet (notice
  `execute` returns a `bool`, not a cycle count — that comes later).
