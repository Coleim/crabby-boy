# 27. Where We Left the PPU, and What's Next

This closing chapter doesn't introduce a new commit — it's a deliberate
pause to take stock, exactly where this book's subtitle promised it
would stop: at the PPU, the one subsystem still earliest in its journey
compared to everything else built so far.

## A quick recap of the whole journey

Looking back at the parts of this book:

- **Part I-II** gave us a CPU that can fetch, decode, and execute (almost)
  every instruction in the Game Boy's instruction set, validated against
  Blargg's `cpu_instrs.gb`.
- **Part III** gave that CPU a real `Bus`/`IOBridge` architecture and ROM
  banking, so bigger games can load at all.
- **Part IV-V** added a Timer, full interrupt dispatch, and a first,
  timing-accurate (but pixel-less) PPU stub — enough to correctly fire
  VBlank once per frame, on schedule.
- **Part VI** built a complete, 4-channel APU, verified against Blargg's
  `dmg_sound` suite.
- **Part VII-VIII** added a joypad stub, and a real terminal UI
  architecture (decoupled from emulation speed), culminating in a live
  CPU/header debug view and a raw VRAM tile decoder.

## What exists for the PPU specifically, right now

Exactly two pieces, from two different chapters:

1. **Chapter 12's minimal PPU**: accurate dot/scanline counting, `LY`
   tracking, and a correctly-timed VBlank interrupt. No pixels, no STAT
   modes, no VRAM access from the PPU itself.
2. **Chapter 26's VRAM tile viewer**: correct 2-bits-per-pixel tile
   decoding, displayed as a raw debug grid — but with no tile *map*, no
   scrolling, no palette application, and entirely disconnected from the
   PPU's own scanline timing.

Between these two pieces, nearly all of the *raw ingredients* for real
background rendering already exist somewhere in this codebase: accurate
timing (Chapter 12) and accurate tile decoding (Chapter 26). What's
missing is the thing that actually combines them into a real frame.

## What comes next: real background rendering

That combination — reading the tile *map* (not just raw tile data),
applying `SCX`/`SCY` scrolling, applying the real `BGP` palette, and
doing all of this in sync with the PPU's actual per-dot timing using the
real **pixel FIFO and fetcher** mechanism — is specifically the subject
of the companion guide:

➡️ **[PPU — Background Rendering Guide](../ppu-background.md)**

That guide picks up exactly where this book leaves off, written in the
same "assume nothing, explain everything" style, and goes deep on:

- The 4 real PPU modes (OAM Scan, Drawing, HBlank, VBlank) and their
  precise dot timing.
- How tile maps, tile data, scrolling, and palettes combine to produce
  one pixel.
- The real pixel FIFO/fetcher state machine real hardware uses, built
  deliberately (not as a shortcut), so window and sprite support can be
  added later without a rewrite.

## Beyond that

Once real background rendering exists, the natural next steps (not yet
written up anywhere at the time of this chapter, but clear from
everything covered so far) are: the window layer (a cheap addition once
the background fetcher exists, since it reuses the same mechanism), and
sprites (a genuinely separate subsystem: a real OAM scan, a second FIFO,
and pixel-priority mixing rules) — plus finishing the joypad (Chapter 21
left real button-to-key mapping as explicitly unbuilt) and STAT-based
interrupts (Chapter 14 left LCD STAT as a vector address with no real
trigger yet).

This is, genuinely, where the project stands today. The rest gets written
as it gets built.
