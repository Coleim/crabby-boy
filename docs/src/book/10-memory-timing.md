# 10. Memory Timing Correctness

Passing `cpu_instrs.gb` (Chapter 4) proves your instructions produce the
*right results*. It does **not** prove they take the *right amount of
time*, or that memory gets touched at the *right moments* during each
instruction. This chapter (`ca64263` → `6e90bef`) is about that second,
much subtler kind of correctness — and about a new category of test ROM
that specifically targets it.

## Why "tick once per instruction" isn't accurate enough

Chapter 9's main loop called `bus.tick(cycles)` once per instruction,
after `cpu.execute` finished. On real hardware, though, a single
instruction isn't an atomic event from the rest of the system's point of
view — it's made of multiple distinct memory accesses (read opcode, maybe
read an operand byte, maybe read/write a memory address), each taking its
own slice of time, with the timer/PPU/APU all continuing to tick
*between* those individual accesses, not just once at the very end. An
instruction that reads memory twice and writes once should let the rest
of the hardware advance in 3 separate steps, not one lump sum at the end.

## The fix: tick on every single bus access

```rust
// src/bus/bus.rs
pub fn internal_tick(&mut self) {
    self.io.tick();
}

pub fn read(&mut self, addr: u16) -> u8 {
    let val = self._read(addr);
    self.internal_tick();
    val
}

pub fn write(&mut self, addr: u16, val: u8) {
    self._write(addr, val);
    self.internal_tick();
}

fn _read(&self, addr: u16) -> u8 { /* the actual match on addr, as before */ }
fn _write(&mut self, addr: u16, val: u8) { /* ditto */ }
```

The real read/write logic moves into private `_read`/`_write` helpers,
and the public `read`/`write` wrap them with an automatic
`internal_tick()` call after every single access. This is a satisfying
refactor: every single place in the whole codebase that touches memory —
every instruction handler, from Chapter 3 onward — now automatically
advances the timer (and later PPU/APU) by the correct amount, without
having to remember to do it manually anywhere. `Bus::tick(cycles)`
(Chapter 9's version, called once per instruction) disappears entirely,
replaced by this call embedded directly in `read`/`write`.

Notice the ripple effect: `read`/`write` now need `&mut self` instead of
`&self`/`&mut self` respectively (`read` wasn't mutating before — now it
has to, to tick internal state). This is exactly the kind of change that
looks small in a diff but touches every call site across the whole
project, because `read` is called from dozens of instruction handlers.

## A new family of test ROMs: `mem_timing`

With per-access ticking in place, a new category of test ROM becomes
meaningful: Blargg's `mem_timing` and `mem_timing-2` suites, specifically
designed to catch exactly this class of bug (an instruction that
*computes* the right result, but touches memory at the wrong moment
relative to the clock). These joined the test suite alongside
`instr_timing.gb` (overall instruction cycle-count correctness).

## Two different ways test ROMs report results

Working through these new test ROMs surfaces something worth
documenting once and reusing forever: not all Blargg-style test ROMs
report results the same way. This project's own notes
(`TEST_ROM_SPECS.MD`) lay out the two conventions found in the wild:

| Convention | Used by | How it reports |
|---|---|---|
| Old (`shell.inc`) | `cpu_instrs`, `instr_timing`, `mem_timing` | Writes each result character to the **serial port** (Chapter 4), ends in an infinite self-loop |
| New | `mem_timing-2`, `dmg_sound`, `oam_bug`, `halt_bug` | Writes a status byte + a fixed signature (`DE B0 61`) + result text directly into **external RAM** at `0xA000`, ends in an infinite self-loop |

```text
$A000     = status (0x80 = running, 0x00 = passed, anything else = error code)
$A001-03  = signature DE B0 61
$A004+    = result text (null-terminated)
```

Both conventions end the same way: an infinite self-loop (`JP $`/`JR $`,
i.e. a jump instruction whose target is itself). That's actually the most
reliable signal of "this test ROM is done" — if `PC` stops changing
between iterations of the main loop, nothing is going to happen that
you haven't already observed, so it's safe to stop and check results.
This is exactly the technique `CrabbyBoy`'s test harness (Chapter 9, and
expanded on in Chapter 11) already leans on.

## What we have now

- Accurate, per-memory-access hardware ticking, instead of an
  end-of-instruction lump sum.
- Both serial-based and RAM-based test ROM result conventions understood
  and documented.
- A growing set of passing timing-sensitive test ROMs:
  `instr_timing`, `mem_timing`, `mem_timing-2` (and its sub-tests:
  read/write/modify timing).

## What's still missing

- `oam_bug` and `dmg_sound` test ROMs are already known about (per
  `TEST_ROM_SPECS.MD`) but not runnable yet — they need OAM/PPU behavior
  and a real APU, respectively, neither of which exist yet at this point.
- This is still "CPU and bus timing," not "PPU timing" — nothing is drawn
  to a screen, and the PPU (Chapter 7) still only stores register bytes.
  Both the PPU (Part V) and the APU (Part VI) still lie ahead.
