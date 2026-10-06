# 7. Filling In Missing Registers

With `IOBridge` in place (Chapter 6), it's now easy to add real storage
behind registers that were previously hardcoded constants or outright
missing. This chapter's commit (`5379233`) does exactly that for the
joypad and — notably — plants the very first `PPU` struct, even though
real graphics are still a long way off.

## `Joypad`: storage first, real buttons later

```rust
// src/hardware/joypad.rs
pub struct Joypad {
    p1: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { p1: 0 }
    }
    pub fn read(&self) -> u8 {
        self.p1
    }
    pub fn write(&mut self, val: u8) {
        self.p1 = val;
    }
}
```

`P1` (register `0xFF00`) is the real name for the joypad register in Game
Boy documentation. At this point it's just a plain byte with no actual
button logic attached — games can write to it and read back whatever they
wrote, which is enough to stop them from getting stuck, without yet
reporting any button presses. Real button handling, including the
slightly unusual "select which group of 4 buttons you're asking about"
protocol, is Chapter 21.

## `PPU`: a struct that exists, but only stores numbers

```rust
// src/hardware/ppu.rs
pub struct PPU {
    lcd_control: u8, // FF40 — LCDC: LCD control
    scy: u8,         // FF42–FF43 — SCY, SCX
    scx: u8,
    lcd_y_coord: u8, // FF44 — LY: LCD Y coordinate [read-only]
    bgp: u8,         // FF47 — BGP
}

impl PPU {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.lcd_y_coord,
            0xFF47 => self.bgp,
            _ => { println!("[PPU] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr); 0x00 }
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => self.lcd_control = val,
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.lcd_y_coord = val,
            0xFF47 => self.bgp = val,
            _ => { /* ... */ }
        }
    }
}
```

This is a deliberately small, honest step: five PPU registers you already
met conceptually in Chapter 0 (`LCDC` — the master "what to draw and how"
control register; `SCY`/`SCX` — background scroll position; `LY` — the
PPU's "current scanline" counter; `BGP` — the background palette), each
just a byte that remembers what was last written to it. Nothing *uses*
these values to draw anything yet — there's no timing, no VRAM access, no
scanline counting. But a real struct now exists to build on top of, which
matters more than it sounds: Chapter 12 (the actual minimal PPU) extends
*this exact struct* rather than starting from scratch.

## Wiring both into `IOBridge`

```rust
pub struct IOBridge {
    joypad: Joypad,
    serial: Serial,
    timer: Timer,
    interrupt_flag: u8,
    audio: APU,
    ppu: PPU,     // $FF40–$FF4B
    key1_spd: u8, // $FF4D — CGB-only, harmless to store anyway
}
```

```rust
pub fn read(&self, addr: u16) -> u8 {
    match addr {
        0xFF00 => self.joypad.read(),
        0xFF01..=0xFF02 => self.serial.read(addr),
        0xFF04..=0xFF07 => self.timer.read(addr),
        0xFF0F => self.interrupt_flag,
        0xFF10..=0xFF26 => self.audio.read(addr),
        0xFF40..=0xFF4B => self.ppu.read(addr),
        0xFF4D => self.key1_spd,
        _ => { println!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr); 0x00 }
    }
}
```

Compare this to Chapter 6's version: the hardcoded `0xFF40 => 0x91`
constant is gone, replaced by `self.ppu.read(addr)` — a real delegation to
a real (if still mostly empty) component. Also notice the fallback for
unmapped addresses changed from `panic!` (Chapter 6) to printing a
warning and returning `0x00` — a small but meaningful robustness
improvement: a game hitting an address we haven't implemented yet no
longer crashes the whole emulator, it just gets a (possibly wrong) zero
and a log line to investigate later.

## What we have now

- A real (if minimal) `PPU` struct and `Joypad` struct, both wired through
  `IOBridge`.
- Five named PPU registers with real backing storage.
- A more forgiving fallback for addresses not yet implemented.

## What's still missing

- `PPU` has no behavior yet — no timing, no scanline progression, no
  interrupt firing, no pixels. Chapter 12 is where it starts actually
  doing something.
- `Joypad` has no concept of actual button state yet — only an
  "echo back whatever was written" byte.
- `STAT` (`0xFF41`, LCD status) isn't listed in the PPU's own read/write
  match yet — it falls through to `IOBridge`'s generic warning path for
  now.
