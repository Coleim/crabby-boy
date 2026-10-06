# 20. Passing Blargg's `dmg_sound` Tests

With all 4 channels built (Part VI so far), this chapter (`bb33d06` →
`c8be255`) is the same kind of milestone Chapter 10 was for CPU timing:
moving from "I implemented what the docs say" to "I verified it against
a trusted, independent test suite" — Blargg's `dmg_sound` ROMs.

## The test suite, one sub-test at a time

```rust
// src/crabby_boy.rs (test module)
cpu_instr_test!(sound_01, "./tests/dmg_sound/01-registers.gb");
cpu_instr_test!(sound_02, "./tests/dmg_sound/02-len ctr.gb");
cpu_instr_test!(sound_03, "./tests/dmg_sound/03-trigger.gb");
cpu_instr_test!(sound_04, "./tests/dmg_sound/04-sweep.gb");
cpu_instr_test!(sound_05, "./tests/dmg_sound/05-sweep details.gb");
cpu_instr_test!(sound_06, "./tests/dmg_sound/06-overflow on trigger.gb");
cpu_instr_test!(sound_07, "./tests/dmg_sound/07-len sweep period sync.gb");
cpu_instr_test!(sound_08, "./tests/dmg_sound/08-len ctr during power.gb");
cpu_instr_test!(sound_09, "./tests/dmg_sound/09-wave read while on.gb");
cpu_instr_test!(sound_10, "./tests/dmg_sound/10-wave trigger while on.gb");
cpu_instr_test!(sound_11, "./tests/dmg_sound/11-regs after power.gb");
cpu_instr_test!(sound_12, "./tests/dmg_sound/12-wave write while on.gb");
cpu_instr_test!(sound_all, "./tests/dmg_sound.gb");
```

Each of these targets one specific area: register read/write
correctness, length-counter edge cases, the exact "trigger" sequence that
happens when a channel is (re)started, sweep behavior and its overflow
edge case, and so on — echoing the "many small, specifically targeted
test ROMs beat one big one" lesson from Chapters 10-11 and 13.

## A real class of bug this suite catches: the "trigger" sequence

Several of the DMG quirks hinted at in Chapters 16/18/19 (sweep's
mode-switch quirk, the length-counter early-clock quirk) are really all
variations on one theme: **what exactly happens at the instant a channel
is triggered** (written to its `NRx4` register with the trigger bit set).
Triggering isn't just "start the channel" — it's a precise sequence:
reload the length counter if it was zero, reload the envelope timer and
volume, reload the sweep shadow register and immediately check for
overflow, reset the waveform position, and (for channel 3 specifically)
sometimes corrupt wave RAM if retriggered at exactly the wrong moment.
Getting this sequence's *order* right, not just each piece in isolation,
is exactly what tests `03-trigger.gb`, `06-overflow on trigger.gb`, and
`10-wave trigger while on.gb` are designed to catch — and exactly the
kind of thing that's nearly impossible to get right by intuition alone,
hence leaning on the test suite rather than guessing.

## `12-wave write while on.gb`: timing matters even for a "simple" write

Channel 3's wave RAM (Chapter 18) can be read and written by the CPU at
almost any time — *except* while the channel is actively playing, during
which real hardware only allows access during one very specific narrow
window per sample, and returns/ignores garbage otherwise. This is the
same "certain hardware state blocks certain memory access" idea as
VRAM/OAM access restrictions in the companion PPU guide — just
discovered here, for wave RAM, via this specific test ROM.

## What we have now

- All 4 channels verified against Blargg's `dmg_sound` suite, sub-test by
  sub-test, as automated `cargo test` entries (Chapter 11's CI runs these
  on every push, too).
- A much more faithful trigger sequence and wave-RAM access timing,
  found and fixed specifically because these tests existed.

## What's still missing

- This closes out Part VI. Sound now works and is verified — but
  everything from here (Parts VII-VIII) was built in parallel with, not
  after, the graphics side of the project; Part IX is where this book
  returns to finish that side.
