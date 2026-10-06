# 9. Timers

The timer is the first hardware peripheral in this book that needs to
track time *independently* of which instruction is currently executing —
it has to keep advancing by exactly the right amount no matter which
opcode just ran. This chapter (`d51d64e`) also introduces the main
emulator loop that ties CPU execution and hardware ticking together for
the first time.

## Four registers, one internal counter

```rust
// src/hardware/timer.rs
pub struct Timer {
    internal_div: u16, // the real 16-bit counter backing DIV
    tima: u8,          // FF05 — TIMA: Timer counter
    tma: u8,           // FF06 — TMA: Timer modulo
    tac: u8,           // FF07 — TAC: Timer control
}
```

- **DIV** (`0xFF04`) — a free-running counter, always ticking, used by
  games for things like random number generation. Reading it returns the
  *top byte* of a larger internal 16-bit counter (`internal_div`); writing
  any value to it resets that internal counter to zero.
- **TIMA** (`0xFF05`) — a counter that increments at a configurable rate,
  and fires the Timer interrupt when it overflows past `0xFF`.
- **TMA** (`0xFF06`) — the value `TIMA` gets reloaded with after
  overflowing (not necessarily 0 — games use this to control exactly how
  often the interrupt fires).
- **TAC** (`0xFF07`) — timer control: a bit to enable/disable `TIMA`
  entirely, plus 2 bits selecting *how fast* it ticks.

## Why `internal_div` is 16 bits, not 8

```rust
pub fn read(&self, addr: u16) -> u8 {
    match addr {
        0xFF04 => (self.internal_div >> 8) as u8,
        // ...
    }
}
```

`DIV` is an 8-bit register from the game's point of view, but the real
hardware actually drives both `DIV` *and* `TIMA`'s variable tick rate off
one shared, wider 16-bit counter running underneath. This matters for the
clever trick in the next section.

## Ticking, one cycle at a time, with falling-edge detection

```rust
pub fn tick(&mut self, cycles: u8) -> bool {
    for _ in 0..cycles {
        let before = self.internal_div;
        self.internal_div = self.internal_div.wrapping_add(1);

        if self.tac & 0b0000_0100 != 0 { // timer enabled?
            let clock_select = self.tac & 0b0000_0011;
            let bit = match clock_select {
                0 => 9,
                1 => 3,
                2 => 5,
                3 => 7,
                _ => unreachable!(),
            };

            let was_set = (before >> bit) & 1 == 1;
            let is_set = (self.internal_div >> bit) & 1 == 1;

            if was_set && !is_set {
                if (self.tima as u16).wrapping_add(1) > 0xFF {
                    self.tima = self.tma; // overflow: reload from TMA
                    return true;          // signal: fire the Timer interrupt
                } else {
                    self.tima = self.tima.wrapping_add(1);
                }
            }
        }
    }
    false
}
```

This is a neat (and very real-hardware-accurate) technique worth slowing
down on. Instead of a separate counter for `TIMA`'s configurable speed,
the implementation watches **one specific bit of the shared 16-bit
counter**, and increments `TIMA` only on that bit's **falling edge** — the
exact moment it flips from `1` to `0` (`was_set && !is_set`). Since a
binary counter's bit `N` flips at a predictable, fixed frequency as the
whole counter increments, picking which bit to watch is equivalent to
picking `TIMA`'s tick frequency — which is exactly what `TAC`'s 2-bit
`clock_select` field controls. This is the same falling-edge-of-a-counter-
bit idea real Game Boy hardware itself actually uses internally, not just
a coincidental implementation choice.

The function ticks **one whole cycle at a time**, in a loop, rather than
computing "how many times would TIMA increment over N cycles" in one
shot — straightforward to write correctly, at the cost of being slower
than a batched approach. That tradeoff (simplicity now, maybe optimize
later if it matters) is a theme you'll see repeatedly in this project.

## The main loop appears: `emulator.rs`

Up to now, `main.rs` has been a quick, throwaway harness. This commit
introduces a real structure, `CrabbyBoy`, with a `run` method that is the
actual fetch-decode-execute-tick loop running for the rest of the book:

```rust
// src/emulator.rs
loop {
    if cpu.stopped {
        println!("CPU STOPPED. Waiting interrupts");
        break;
    }
    if cpu.halt {
        bus.tick(4);
        let ie = bus.get_ie();
        let if_flag = bus.get_io().get_if();
        if (ie & if_flag) != 0 {
            cpu.halt = false;
        }
        continue;
    }

    match cpu.execute(&mut bus) {
        Some(tick) => {
            bus.tick(tick);
            cpu.handle_interrupts(&mut bus);
        }
        None => panic!("Error in getting the cycles"),
    }
}
```

Two changes worth highlighting against earlier chapters:

- `CPU::execute` now returns `Option<u8>` (how many cycles the
  instruction took), not a plain `bool`. This is what finally lets
  `bus.tick(tick)` advance the timer (and, later, every other hardware
  peripheral) by exactly the right amount after every single instruction
  — tying CPU execution and hardware timing together for the first time
  in this project.
- Even while halted, `bus.tick(4)` keeps running every loop iteration —
  because real hardware doesn't freeze the timer/PPU/APU just because the
  CPU itself is halted; only CPU instruction execution pauses.

## Tests move into real Rust tests

```rust
#[cfg(test)]
macro_rules! cpu_instr_test {
    ($name: ident, $path: expr) => {
        #[test]
        fn $name() {
            let mut crabby = CrabbyBoy::new();
            assert_eq!(crabby.run($path), Ok(()));
        }
    };
}
```

Instead of manually editing `main.rs` to point at a different test ROM
and eyeballing printed output (what we've been doing since Chapter 4),
test ROMs now run as real, automated `cargo test` tests, each one just a
one-line macro invocation naming a ROM file. `CrabbyBoy::run` itself
detects "Passed"/"Failed" text in the serial output (Chapter 4) and turns
it into a proper `Result`. This is a meaningful quality-of-life jump, and
it's the foundation Chapter 11 builds a full CI pipeline on top of.

## What we have now

- A real `Timer` with falling-edge-accurate `DIV`/`TIMA`/`TMA`/`TAC`
  behavior, including firing the Timer interrupt on overflow.
- A proper main loop (`CrabbyBoy::run`) that ticks hardware after every
  instruction and handles `HALT` correctly waking up on a pending
  interrupt.
- Automated `#[test]` functions for test ROMs, replacing manual
  `main.rs` editing.
- New test ROMs for instruction and memory-access timing, used in the
  next chapter.

## What's still missing

- `cpu.handle_interrupts` is called here but real interrupt *dispatch*
  (jumping to the right vector, pushing `PC`, clearing `IME`) is only
  properly covered in Chapter 14 — this chapter focuses on the timer
  itself and the loop structure around it.
- Most of the `cpu_instr_test!` invocations are still commented out at
  this point (`read_timing` is the only one active) — memory timing
  correctness (why that matters, and fixing it) is Chapter 10.
