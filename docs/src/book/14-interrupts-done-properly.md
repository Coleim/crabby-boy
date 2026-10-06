# 14. Interrupts Done Properly

With the Timer (Chapter 9) and the PPU's VBlank (Chapter 12) both now
able to set bits in `IF`, it's finally time to implement what actually
happens when an interrupt fires: **dispatch** — pausing whatever the CPU
was doing and jumping to a fixed handler address. This chapter covers
`5af7b4b` and the test-framework cleanup in `7e246c4`.

## The dispatch logic

```rust
pub fn handle_interrupts(&mut self, bus: &mut Bus) {
    if self.ime == false {
        return; // interrupts globally disabled — do nothing
    }
    let ie = bus.get_ie();
    let iflag = bus.get_io().get_if();
    let triggered = ie & iflag; // enabled AND pending

    if triggered == 0 {
        return;
    }

    let bit = triggered.trailing_zeros() as u8;
    let mask = 1 << bit;
    bus.clear_if(mask); // mark this one as handled
    self.ime = false;   // disable further interrupts until RETI

    // 2 wait cycles are executed
    bus.internal_tick();
    bus.internal_tick();

    // Push current PC onto the stack, so RETI can come back here
    self.sp = self.sp.wrapping_sub(2);
    self.write16bytes(bus, self.sp, self.pc);

    bus.internal_tick();

    match bit {
        0 => self.pc = 0x0040, // VBlank
        1 => self.pc = 0x0048, // LCD STAT
        2 => self.pc = 0x0050, // Timer
        3 => self.pc = 0x0058, // Serial
        4 => self.pc = 0x0060, // Joypad
        _ => unreachable!(),
    }
}
```

Step by step, matching the conceptual summary from Chapter 0:

1. **`IME` gate** — if the CPU-internal master switch is off, nothing
   happens, no matter what's pending. This is also exactly the condition
   the HALT bug (Chapters 5 and 13) cares about.
2. **`IE & IF`** — only interrupts that are *both* individually enabled
   (`IE`, `0xFFFF`) *and* currently pending (`IF`, `0xFF0F`) count.
3. **Priority via `trailing_zeros()`** — if multiple interrupt bits are
   set simultaneously, the Game Boy always services the **lowest-numbered
   bit first** (VBlank beats LCD STAT beats Timer beats Serial beats
   Joypad). `u8::trailing_zeros()` is a neat one-liner for "index of the
   lowest set bit" — exactly the priority order needed, for free.
4. **Clear that one `IF` bit**, and **clear `IME`** — while handling this
   interrupt, no other interrupt (not even a higher-priority one) can
   dispatch on top of it, until the handler explicitly re-enables
   interrupts (typically by ending with the `RETI` instruction from
   Chapter 4, which restores `IME`).
5. **Push `PC` onto the stack** — exactly like the `CALL` instruction
   does, so that once the handler finishes, it can resume exactly where
   normal execution left off.
6. **Jump to a fixed vector address** — each interrupt type has one
   fixed, hardcoded entry point (`0x0040` for VBlank, and so on);
   there's no decoding involved, these addresses are a hardware constant,
   the same way `0x0100` is always the cartridge entry point (Chapter 3).

## Why the extra `bus.internal_tick()` calls matter

Notice the two internal ticks before pushing `PC`, and one more after.
Dispatching an interrupt isn't instantaneous on real hardware — it takes
a fixed number of cycles (5 M-cycles total, in this implementation's
accounting: 2 "wait" cycles, 2 for pushing the 2-byte `PC` onto the stack
—matching `write16bytes`'s own internal ticking from Chapter 10's
per-access tick model — plus 1 more). Skipping these would make interrupt
dispatch "free" in terms of timing, which would throw off any test ROM
(like `interrupt_time.gb`, added in this very commit) checking exactly
how many cycles pass around an interrupt firing.

## What we have now

- Full interrupt dispatch: priority ordering, `IME`/`IE`/`IF` gating,
  correct stack push, correct vector jump, correct cycle cost.
- `interrupt_time.gb` added as a test ROM specifically to validate this
  timing.
- Both of this project's two interrupt sources so far (Timer, VBlank)
  now actually *do something* when they fire, instead of just setting a
  bit nobody reacts to.

## What's still missing

- Only 2 of the 5 interrupt types (VBlank, Timer) have a real hardware
  source behind them right now — Serial and Joypad interrupts exist as
  vector addresses in this `match`, but nothing yet sets their `IF` bits.
  The Joypad one gets wired up properly in Chapter 21.
- LCD STAT interrupts specifically depend on PPU "mode" tracking, which
  (per Chapter 12) doesn't exist yet.
- This closes out Part IV. From here, the project's history branches into
  largely independent subsystems built side by side: sound (Part VI, next),
  input (Part VII), and a terminal UI (Part VIII) — before this book
  circles back to the PPU one final time in Part IX.
