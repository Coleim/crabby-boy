# 6. Splitting into CPU / Bus / Hardware

After the opcode grind (Part II), the project had everything crammed
under `src/cpu/`: the CPU itself, the bus, the cartridge header, the
serial port, a timer, even the beginnings of audio. That stops scaling
once more hardware pieces (joypad, PPU, more APU channels...) are about to
join. This chapter's commit (`db7f453`) is a pure reorganization — almost
no new behavior, just a much better shape for everything that follows.

## The new module layout

```text
src/
  cpu/        — the CPU itself (registers, opcodes, header parsing)
  bus/        — the memory bus, and a new IOBridge
  hardware/   — timer, serial, APU, (soon: joypad, PPU)
```

The guiding idea: `cpu/` should only know about *executing instructions*.
Everything an instruction might read or write — memory, I/O registers,
timers, sound — belongs under `bus/` and `hardware/` instead. This is a
very common shape for emulators in general: one module that's "the thing
executing code," and a separate layer modeling "everything the code can
observe or affect."

## A new concept: `IOBridge`

Chapter 4 already introduced the memory map and the idea that
`0xFF00`–`0xFF7F` is a block of **I/O registers**, not real memory. Up to
now, `Bus` handled a couple of those addresses directly (just the serial
port). As more hardware components need their own slice of that address
range, cramming all of their logic into `Bus::read`/`Bus::write` directly
would make `Bus` enormous and tightly coupled to every peripheral. So a
new struct appears specifically to own *just* that region:

```rust
// src/bus/iobridge.rs
pub struct IOBridge {
    serial: Serial,
    timer: Timer,       // $FF04-$FF07 — Timer and divider
    interrupt_flag: u8, // $FF0F — IF: Interrupt flag
    audio: APU,         // $FF10-$FF26 — Audio
}

impl IOBridge {
    pub fn tick(&mut self, cycles: u8) {
        if self.timer.tick(cycles) {
            self.interrupt_flag |= 0b0000_0100;
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01..=0xFF02 => self.serial.read(addr),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupt_flag,
            0xFF10..=0xFF26 => self.audio.read(addr),
            0xFF40 => 0x91, // LCDC — hardcoded placeholder for now
            0xFF41 => 0x85, // STAT — hardcoded placeholder for now
            0xFF44 => 0x00, // LY — hardcoded placeholder for now
            // ...
            _ => std::panic!("[IOREG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01..=0xFF02 => self.serial.write(addr, val),
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupt_flag = val,
            0xFF10..=0xFF26 => self.audio.write(addr, val),
            0xFF40..=0xFF4B => {} // PPU registers — silently ignored for now
            _ => {}
        }
    }
}
```

Notice the pattern taking shape: `IOBridge` is itself a little "mini
bus" — it owns several hardware peripherals and routes each I/O address to
whichever one of them actually owns it. `Bus` (Chapter 4) will hand off
the entire `0xFF00..=0xFF7F` range to `IOBridge::read`/`write`, the same
way the CPU hands off `0x8000..=0x9FFF` to `Bus`. This "delegate to a
sub-component responsible for one address range" pattern repeats at every
level of this emulator, all the way down.

## Hardcoded register values: an honest placeholder

Notice `0xFF40 => 0x91` (LCDC) and `0xFF44 => 0x00` (LY) are just
constants, not backed by any real PPU state yet — there is no PPU struct
at all at this exact point in history. Many games read these registers
just to check "is the screen in a safe state to update graphics," and a
plausible-looking constant is often enough to let a game's boot sequence
continue, long before any real graphics chip exists to back it up. This
is a useful general technique when bootstrapping an emulator: fake a
register with a reasonable fixed value first, replace it with the real
thing later (Part V starts that replacement for LCDC/STAT/LY).

## What we have now

- Three clearly separated concerns: `cpu/`, `bus/`, `hardware/`.
- `IOBridge`, a new routing layer specifically for I/O registers, already
  wired to the timer, serial port, and the first APU scaffold.
- Placeholder PPU-related register values, good enough to keep games
  running without a real PPU existing yet.

## What's still missing

- No `Joypad`, no real `PPU` struct yet — those arrive next chapter.
- `IOBridge::tick` only advances the timer so far; nothing else ticks
  alongside the CPU yet.
- Unmapped reads still `panic!` outright rather than returning a safe
  default — fine for now, since every ROM we're testing against is
  well-behaved, but worth remembering as a rough edge.
