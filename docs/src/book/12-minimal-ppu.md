# 12. Scanlines 101 & a Minimal PPU

This is the chapter this whole book has been building toward (and,
honestly, where a new companion guide picks up afterward — see Part IX).
This commit (`624949c`) gives the `PPU` struct from Chapter 7 its first
real behavior: counting time and firing an interrupt. Still **no pixels
yet** — that's deliberate, and explained below.

## A reminder: how the real PPU spends its time

(If this feels new, Chapter 0 introduced the vocabulary — PPU, VRAM — at a
glance; here's the timing model specifically.)

The screen is 160×144 pixels, drawn one line at a time, 60 times a
second. A full frame takes exactly **70,224 "dots"** (the PPU's own clock
unit, 4 dots per CPU M-cycle) — broken down as **154 scanlines × 456 dots
each**. Of those 154 scanlines, only the first 144 are actually visible;
the remaining 10 are a pause called **VBlank** (vertical blank), used
historically to give old CRT displays time for their electron beam to
physically return to the top-left corner, and used by Game Boy games as a
safe window to update graphics data without tearing.

## Counting dots, one tick at a time

```rust
// src/hardware/ppu.rs
pub struct PPU {
    lcd_control: u8,
    scy: u8,
    scx: u8,
    lcd_y_coord: u8, // FF44 — LY
    bgp: u8,
    dots: u16,       // T-cycle counter
}

pub fn tick(&mut self) -> bool {
    self.dots += 4;
    if self.dots >= 456 {
        self.dots -= 456;
        self.lcd_y_coord += 1;
        if self.lcd_y_coord >= 154 {
            self.lcd_y_coord = 0;
        }
        if self.lcd_y_coord == 144 {
            return true;
        }
    }
    false
}
```

Compare this directly to the Timer's `tick` from Chapter 9 — same shape,
same idea: an internal counter advances every time this is called, and
when it crosses a threshold (456 dots), something observable happens
(here: `LY`, the current-scanline register from Chapter 7, increments).
When `LY` reaches 144, that's the exact moment VBlank begins — and
`tick` returns `true` to signal it, exactly once per frame.

`self.dots += 4` matches how this function gets called: once per CPU
M-cycle, each worth 4 dots — the same per-access ticking granularity
Chapter 10 built for the timer, now driving the PPU too.

## Wiring the VBlank interrupt

```rust
// src/bus/iobridge.rs
pub fn tick(&mut self) {
    if self.timer.tick() {
        self.interrupt_flag |= 0b0000_0100; // Timer interrupt
    }
    if self.ppu.tick() {
        self.interrupt_flag |= 0b0000_0001; // VBlank interrupt
    }
    self.audio.tick();
}
```

The exact same pattern as the Timer interrupt from Chapter 9: `tick()`
returns `true` exactly when something interrupt-worthy happened, and the
caller OR's the matching bit into `interrupt_flag` (`IF`, `0xFF0F`).
VBlank is interrupt bit 0 — the very first and, on real hardware, by far
the most commonly used interrupt, since it's how games know it's safe to
start the next frame's graphics updates.

## Two small `IF`/`IE` correctness fixes, in passing

```rust
0xFF0F => self.interrupt_flag | 0b1110_000, // on read: top 3 bits always read as 1
```
```rust
0xFF0F => self.interrupt_flag = val & 0b0001_1111, // on write: only the low 5 bits are real
```

`IF` is only a 5-bit register in hardware terms (one bit per interrupt
type: VBlank, STAT, Timer, Serial, Joypad) — the top 3 bits don't exist as
real storage and always read back as `1`. Now that a *second* real
interrupt source (VBlank) exists alongside the Timer, these details
actually start to matter for test ROMs that inspect `IF` precisely, so
they get tightened up here.

## Why no pixels yet — and that's genuinely fine

It would be reasonable to expect "the PPU chapter" to end with something
appearing on a screen. This one doesn't, and that's an intentional,
honest snapshot of how the project was actually built: get the *timing*
skeleton right and provably correct first (dots → scanlines → VBlank →
interrupt), *before* spending effort on tile decoding, scrolling, and
palettes. A `PPU` that correctly ticks `LY` and fires `VBlank` on time is
already enough to:

- Let games progress past boot sequences that wait for VBlank before
  continuing.
- Give `halt_bug.gb` (Chapter 5) and `interrupt_time.gb` a real,
  non-Timer interrupt source to test against, instead of only the
  synthetic conditions used so far.
- Provide a stable foundation (`dots`, `lcd_y_coord`, the register
  layout) that later chapters build real rendering on top of, rather than
  rewriting from scratch.

## What we have now

- A `PPU` that accurately tracks scanlines and dots, matching the real
  70,224-dots-per-frame timing model.
- A working VBlank interrupt, firing exactly once per frame at the right
  moment.
- Slightly more accurate `IE`/`IF` register read/write masking.

## What's still missing

- No STAT (`0xFF41`) mode tracking yet — real hardware exposes which of 4
  PPU modes (OAM scan / drawing / HBlank / VBlank) is currently active,
  and this isn't modeled at all yet.
- No VRAM or OAM access from the PPU at all — it can't read tile data,
  because it doesn't look at `vram`/`oam` (still owned by `Bus`) in any
  way.
- **No pixels, no framebuffer, no window, no sprites.** All of that is
  deliberately deferred — Part IX revisits the PPU once more at the end of
  this book, and the real pixel-rendering work (the pixel FIFO/fetcher
  mechanism) is the subject of the separate
  [PPU Background Rendering Guide](../ppu-background.md).
