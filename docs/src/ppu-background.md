# PPU — Background Rendering Guide

This guide explains, step by step, how to make your PPU draw the
background layer onto a 160x144 pixel screen. We go slowly. Nothing is
assumed. By the end you'll understand exactly what data lives where, and
how it turns into pixels.

> **Scope of this guide: background only.** No window, no sprites. Those
> come later, on top of what you build here.

Official reference docs (Pan Docs) are linked throughout. When in doubt,
go read the linked page — this guide simplifies things to make them easier
to learn, the official docs are the ground truth.

## 1. The big picture

Every 1/60th of a second, the Game Boy draws one full image (a "frame") to
the screen. The screen is 160 pixels wide and 144 pixels tall.

The image is NOT drawn all at once. It's drawn **one horizontal line at a
time**, from the top (line 0) to the bottom (line 143), left to right on
each line. This matches how old CRT screens physically worked: a beam
scans across, then down, repeatedly.

A chip called the **PPU** (Pixel Processing Unit) is responsible for this.
It runs continuously, in lockstep with the CPU, counting time in units
called **dots**. A full line takes exactly **456 dots**. There are **154
lines** total per frame (not 144 — more on this in a moment).

Reference: <https://gbdev.io/pandocs/Rendering.html>

### Why 154 lines, not 144?

The screen only has 144 visible rows. But the PPU still spends time
pretending to draw 10 more invisible rows (lines 144 to 153) after the
real image is done. This extra time is called **VBlank** ("vertical
blank"), and it exists so the game has a safe window to update things
before the next frame starts drawing. Nothing is drawn to the screen
during these 10 lines — think of it as a pause.

So: 144 real lines + 10 "pause" lines = 154 lines per frame.

## 2. The 4 modes, and what happens during each line

For each of the first 144 lines, the PPU goes through 3 phases, always in
the same order. Then, once per frame, after line 143, it spends 10 whole
lines in a 4th phase (VBlank). Here's the timeline for ONE visible line
(say, line 0):

```text
dot:     0 ────────────── 80 ────────────── 252 ──────────────── 456
mode:    |   Mode 2        |     Mode 3       |      Mode 0        |
         |   OAM Scan      |     Drawing      |      HBlank        |
         |   (80 dots)     |   (172 dots)     |   (204 dots)       |
```

- **Mode 2 — OAM Scan** (first 80 dots of the line): the PPU would
  normally check which sprites appear on this line. **We are not doing
  sprites in this guide**, so for us this phase currently does nothing —
  it's just a placeholder. Just know it exists and takes 80 dots.

- **Mode 3 — Drawing** (next 172 dots, in our simplified version): this is
  the important phase. This is where the PPU actually figures out the
  color of each of the 160 pixels on this line and writes them to the
  screen. **This is the core of what this guide teaches you to build.**

  > Note: on real hardware, Mode 3's length varies (172 to 289 dots)
  > depending on sprites, the window layer, and scroll position. We are
  > deliberately using a **fixed 172 dots** to keep things simple. This is
  > a known simplification — see [section 7](#7-what-this-guide-deliberately-does-not-cover)
  > for what real hardware does differently.

- **Mode 0 — HBlank** (remaining dots until 456): nothing happens, this is
  just dead time so the whole line adds up to exactly 456 dots.

After all 144 visible lines, we get:

- **Mode 1 — VBlank** (10 lines x 456 dots = 4560 dots): the PPU is idle,
  the frame is considered finished and ready to show. An interrupt fires
  here so the game code knows a new frame just completed.

Reference (mode durations table):
<https://gbdev.io/pandocs/Rendering.html#ppu-modes>

## 3. Where does the picture data actually come from?

This is the part that trips people up: **VRAM does not contain pixels.**
It contains two different kinds of things, and you need both to produce a
pixel:

1. **Tile data**: the actual shapes (8x8 pixel images), stored in a
   compressed 2-bits-per-pixel format.
2. **Tile map**: a 32x32 grid of numbers, where each number says "put tile
   #N here." It's like a mosaic instruction sheet: "tile 5 goes top-left,
   tile 12 goes next to it," etc.

So to find the color of one pixel, you must:

1. Figure out which tile (from the tile MAP) covers that pixel.
2. Look up that tile's actual image data (from the tile DATA area).
3. Extract the one pixel you need from that 8x8 tile image.

Let's go through each piece in detail.

### 3.1 The tile map

VRAM contains **two** possible tile maps, at fixed addresses:

- `$9800`–`$9BFF`
- `$9C00`–`$9FFF`

Each is 32x32 = 1024 bytes. Each byte is a **tile index** (0–255), telling
you which tile from the tile data area to draw at that grid position.

Since each tile is 8x8 pixels, a full 32x32 tile map represents a picture
that is 256x256 pixels — much bigger than the 160x144 screen. Only a
160x144 window into this larger picture is shown at any time (more on this
in [section 3.4](#34-scrolling-scx--scy), scrolling).

Which of the two maps is used for the background is controlled by **bit 3**
of the LCDC register (`$FF40`):

- bit 3 = 0 → use `$9800`
- bit 3 = 1 → use `$9C00`

Reference: <https://gbdev.io/pandocs/Tile_Maps.html>
Reference (LCDC bits): <https://gbdev.io/pandocs/LCDC.html>

### 3.2 The tile data (the actual pixel shapes)

Each tile is 8x8 pixels, but a Game Boy pixel isn't a full color — it's
just a **2-bit number (0, 1, 2, or 3)**, called a "color index." What
color that number actually means on screen is decided later by a palette
([section 3.5](#35-the-palette-bgp)).

Each tile is stored as **16 bytes**: 2 bytes per row, 8 rows. Why 2 bytes
per row of 8 pixels? Because each pixel needs 2 bits, and 8 pixels x 2 bits
= 16 bits = exactly 2 bytes. The two bytes work together: for a given
pixel at column `c` (0 = leftmost, 7 = rightmost):

```text
bit_position = 7 - c          // bit 7 is the leftmost pixel, bit 0 is rightmost
low_bit  = (first_byte  >> bit_position) & 1
high_bit = (second_byte >> bit_position) & 1
color_index = (high_bit << 1) | low_bit   // combine into a 0-3 value
```

This is exactly the decoding logic already written in
`src/display/vram_registers.rs` (`generate_buffer`), which you already got
working for the VRAM tile viewer. Good news: you don't need to invent this
part again, just reuse the same formula.

Here it is wrapped into a reusable function — given a tile's **resolved
data address** (section 3.3 below computes this) and a **screen
position** to draw it at, it decodes all 8 rows of that tile and writes
the shaded pixels straight into the frame buffer:

```rust
/// Decodes one 8x8 tile's pixel data and writes it into the frame buffer
/// at the given screen position, applying the BGP palette.
fn draw_tile(
    vram: &[u8],
    tile_data_addr: usize,
    bgp: u8,
    frame_buffer: &mut [u8],
    screen_x: usize,
    screen_y: usize,
) {
    for pixel_row in 0..8 {
        let lo = vram[(tile_data_addr + pixel_row * 2) - 0x8000];
        let hi = vram[(tile_data_addr + pixel_row * 2 + 1) - 0x8000];

        for pixel_col in 0..8 {
            let bit = 7 - pixel_col;
            let color_index = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
            let shade = (bgp >> (color_index * 2)) & 0b11; // section 3.5

            let x = screen_x + pixel_col;
            let y = screen_y + pixel_row;
            frame_buffer[y * 160 + x] = shade;
        }
    }
}
```

Notice this function doesn't know or care *why* it's being called — it
just draws one tile at one screen position. That's deliberate: section 4
calls it from a naive full-screen loop, and it's just as usable later
from inside the real per-dot fetcher (section 5).

Reference: <https://gbdev.io/pandocs/Tile_Data.html>

### 3.3 Finding a tile's address from its index

Here's a quirk: there are **two different ways** to interpret a tile
index number into a memory address, and the game picks which one via
**bit 4 of LCDC**:

- **If LCDC bit 4 = 1** ("unsigned" mode): tile index is just used as-is,
  0 to 255, starting from address `$8000`.

  ```text
  address = 0x8000 + tile_index * 16
  ```

- **If LCDC bit 4 = 0** ("signed" mode): tile index is treated as a signed
  number (-128 to 127!), and the base address is `$9000` instead.

  ```text
  address = 0x9000 + (tile_index_as_signed_byte) * 16
  ```

  So here, an index of 0 still means `$9000`, but an index of 255 (which
  as a signed byte is -1) means `$9000 - 16 = $8FF0`.

Why does this exist? Historical hardware reasons — both modes overlap in
the memory range `$8800`–`$97FF`, which is shared. You don't need to know
why, just that you must check LCDC bit 4 before computing the address.

As a reusable function:

```rust
/// Resolves a tile index to its tile data address, per LCDC bit 4.
fn resolve_tile_data_addr(tile_index: u8, lcdc: u8) -> usize {
    if lcdc & 0b0001_0000 != 0 {
        // Unsigned addressing mode
        0x8000 + (tile_index as usize) * 16
    } else {
        // Signed addressing mode
        (0x9000_i32 + (tile_index as i8 as i32) * 16) as usize
    }
}
```

Reference: <https://gbdev.io/pandocs/Tile_Data.html> — see "Addressing
modes."

### 3.4 Scrolling (SCX / SCY)

Remember the tile map represents a 256x256 picture, but the screen only
shows 160x144 of it. The **SCX** (`$FF43`) and **SCY** (`$FF42`) registers
say "which pixel of that big 256x256 picture appears at the screen's
top-left corner?"

So for screen pixel `(x, y)`, the corresponding pixel in the big 256x256
background picture is:

```text
bg_x = (x + SCX) % 256
bg_y = (y + SCY) % 256
```

The `% 256` (modulo) is important: if scrolling pushes you past the edge
of the 256x256 picture, it **wraps around** to the other side, like a
looping wallpaper pattern, instead of just showing blank space.

Reference: <https://gbdev.io/pandocs/Scrolling.html>

### 3.5 The palette (BGP)

Once you have a color index (0–3) for a pixel, that's still not a final
color — it's an index into a 4-entry lookup table called **BGP**
(`$FF47`, "BG Palette"). BGP is one byte that packs 4 separate 2-bit
values, one per possible color index:

```text
BGP byte:   bit7 bit6 | bit5 bit4 | bit3 bit2 | bit1 bit0
             shade for    shade for   shade for   shade for
             index 3      index 2     index 1     index 0
```

To get the final shade (also 0–3, but now meaning an actual gray shade:
0=white, 3=black, roughly) for a given `color_index`:

```text
shade = (BGP >> (color_index * 2)) & 0b11
```

Reference: <https://gbdev.io/pandocs/Palettes.html>

## 4. Putting it together: the simple (but wrong) way

Just to build intuition, here's the "obvious but not how hardware really
does it" way to render an entire frame: loop over every **tile** that
fits on screen (not every pixel), look up its index in the tile map,
resolve and draw it, using the two functions from sections 3.2 and 3.3:

```rust
let tile_map_base: usize = 0x9800; // section 3.1 — assumes LCDC bit 3 = 0

for tile_row in 0..18 {       // 144 / 8 = 18 tile rows fit on screen
    for tile_col in 0..20 {   // 160 / 8 = 20 tile columns fit on screen
        let offset = tile_row * 32 + tile_col;
        let tile_index = vram[(tile_map_base + offset) - 0x8000];
        let tile_data_addr = resolve_tile_data_addr(tile_index, lcdc); // section 3.3

        draw_tile(vram, tile_data_addr, bgp, &mut frame_buffer, tile_col * 8, tile_row * 8); // section 3.2
    }
}
```

A few things worth noting about this example:

- `18` and `20`, not `144`/`160` — this loops over **tiles that fit on
  screen**, not individual pixels. Each iteration handles an entire 8x8
  block at once, via `draw_tile`.
- This ignores `SCX`/`SCY` scrolling entirely (assumes both are 0, and
  always reads tile map 1) — fine as a true "naive first scene" starting
  point, but you'd revisit this (applying section 3.4's formulas before
  computing `tile_row`/`tile_col`) once scrolling matters.
- For a single static frame, this produces the exact same pixels as a
  pixel-by-pixel loop that recomputes `tile_row = y / 8` /
  `tile_col = x / 8` for every one of the 160x144 pixels individually —
  it's just less redundant, since each tile's lookup/decode happens once
  here instead of 64 times.

This produces a correct image! But it has one deeper problem: it computes
the **entire frame** all at once, which is **not what the real PPU
hardware does**. Real hardware produces pixels one at a time, gradually,
line by line, during each line's Mode 3 (172 dots). We want to build the
real mechanism, because:

- It's genuinely not much harder.
- It's the actual architecture you'll need later for window/sprites.
- It matches Pan Docs, so you can cross-reference your code against the
  spec directly.

So let's build the real version instead.

## 5. The real mechanism: the Pixel FIFO and the Fetcher

Real hardware uses two cooperating pieces, running only during Mode 3:

- **The Fetcher**: a small step-by-step machine that reads the tile map
  and tile data ([sections 3.1–3.3](#31-the-tile-map)) and prepares a row
  of 8 pixels at a time.
- **The FIFO** ("First In First Out" queue): a buffer that holds up to 16
  pending pixels. The fetcher pushes batches of 8 pixels into it. Every
  dot, if the FIFO has pixels waiting, one gets popped off and drawn to
  the screen.

Think of it like a conveyor belt: the fetcher is a worker putting pixels
onto the belt in batches of 8, and the screen is pulling one pixel off the
belt per dot. As long as the worker keeps up (doesn't let the belt run
empty), pixels flow out steadily.

Reference: <https://gbdev.io/pandocs/pixel_fifo.html>

### 5.1 The Fetcher's 5 steps

The fetcher repeats this cycle of 5 steps, forever, during Mode 3. Here's
the state it needs, and a sketch of each step as code, reusing
`resolve_tile_data_addr` from section 3.3:

```rust
enum FetcherStep { GetTile, GetTileDataLow, GetTileDataHigh, Sleep, Push }

struct Ppu {
    bg_fifo: VecDeque<u8>,       // pending color indices (0-3)
    fetcher_step: FetcherStep,
    fetcher_tile_col: u8,        // which tile map column we're fetching (0-31)
    tile_index: u8,              // scratch: tile index just read
    tile_data_lo: u8,            // scratch: low byte of the tile row
    tile_data_hi: u8,            // scratch: high byte of the tile row
    lcd_x: u8,                   // next screen column to output (0-159)
    lcd_y_coord: u8,             // current scanline (LY)
    scx: u8,
    scy: u8,
    lcdc: u8,
    bgp: u8,
    // ... dots, mode, etc. from the minimal PPU you already have
}

fn tick_fetcher(ppu: &mut Ppu, vram: &[u8]) {
    match ppu.fetcher_step {
        FetcherStep::GetTile => {
            // section 3.1: which tile map, which row/col
            let tile_map_base: usize = if ppu.lcdc & 0b0000_1000 != 0 { 0x9C00 } else { 0x9800 };
            let tile_row = ((ppu.lcd_y_coord as u16 + ppu.scy as u16) % 256) / 8;
            let offset = tile_row as usize * 32 + ppu.fetcher_tile_col as usize;
            ppu.tile_index = vram[(tile_map_base + offset) - 0x8000];
            ppu.fetcher_step = FetcherStep::GetTileDataLow;
        }
        FetcherStep::GetTileDataLow => {
            let addr = resolve_tile_data_addr(ppu.tile_index, ppu.lcdc); // section 3.3
            let row = (ppu.lcd_y_coord as u16 + ppu.scy as u16) % 8;
            ppu.tile_data_lo = vram[(addr + row as usize * 2) - 0x8000];
            ppu.fetcher_step = FetcherStep::GetTileDataHigh;
        }
        FetcherStep::GetTileDataHigh => {
            let addr = resolve_tile_data_addr(ppu.tile_index, ppu.lcdc);
            let row = (ppu.lcd_y_coord as u16 + ppu.scy as u16) % 8;
            ppu.tile_data_hi = vram[(addr + row as usize * 2 + 1) - 0x8000];
            ppu.fetcher_step = FetcherStep::Sleep;
        }
        FetcherStep::Sleep => {
            ppu.fetcher_step = FetcherStep::Push;
        }
        FetcherStep::Push => {
            if ppu.bg_fifo.is_empty() {
                // same 2bpp decode formula as draw_tile (section 3.2),
                // just pushed into the FIFO instead of written directly
                for pixel_col in 0..8 {
                    let bit = 7 - pixel_col;
                    let color_index = ((ppu.tile_data_hi >> bit) & 1) << 1 | ((ppu.tile_data_lo >> bit) & 1);
                    ppu.bg_fifo.push_back(color_index);
                }
                ppu.fetcher_tile_col = (ppu.fetcher_tile_col + 1) & 0x1F;
                ppu.fetcher_step = FetcherStep::GetTile;
            }
            // if the FIFO wasn't empty, stay on this step and retry next dot
        }
    }
}
```

Each of `GetTile`, `GetTileDataLow`, `GetTileDataHigh`, and `Sleep` takes 2
dots (so in practice you'd only actually advance the state machine every
other dot — simplified here for clarity). `Push` is attempted every dot
until it succeeds. So: 2+2+2+2 = 8 dots minimum per tile, plus possibly
extra dots stuck retrying `Push` if the FIFO hasn't emptied yet.

### 5.2 Every dot, independently: try to output a pixel

At the same time as the fetcher is doing its thing, **every single dot**
during Mode 3, this also happens:

```rust
fn output_pixel(ppu: &mut Ppu, frame_buffer: &mut [u8]) {
    if let Some(color_index) = ppu.bg_fifo.pop_front() {
        let shade = (ppu.bgp >> (color_index * 2)) & 0b11; // section 3.5
        let y = ppu.lcd_y_coord as usize;
        let x = ppu.lcd_x as usize;
        frame_buffer[y * 160 + x] = shade;
        ppu.lcd_x += 1;
    }
    // if the FIFO was empty, do nothing this dot — the screen waits for
    // the fetcher to catch up
}
```

Once `lcd_x` reaches 160, this line's drawing is done — switch to Mode 0
(HBlank) immediately, regardless of what the fetcher is mid-way through
doing.

### 5.3 Why bother with this instead of the simple way?

Because this is genuinely how the hardware works, and the "simple way"
from [section 4](#4-putting-it-together-the-simple-but-wrong-way) is a
shortcut that happens to produce the same final picture **only because
we're not yet doing anything that changes mid-line** (like switching to
the window layer partway across a row, or mixing in sprites). Once you
want to support those features, you need the real fetcher+FIFO machinery
anyway — so you might as well build it now and avoid a rewrite later.

## 6. What you need to add to your code (checklist)

On your `PPU` struct, add:

- `frame_buffer: [u8; 160 * 144]` — one shade (0–3) per pixel, storing
  pixel `(x, y)` at index `y * 160 + x`.
- `bg_fifo: VecDeque<u8>` — holds pending color indices (0–3), max size 16
  in theory (we'll never actually get close to that limit in this simple
  version).
- `fetcher_step` — a small enum: `GetTile`, `GetTileDataLow`,
  `GetTileDataHigh`, `Sleep`, `Push`.
- `fetcher_tile_col: u8` — which column (0–31) of the tile map the fetcher
  is currently working on for this line.
- `lcd_x: u8` — which screen column (0–159) is about to be written next.
- Scratch fields to remember data between fetcher steps: `tile_index: u8`,
  `tile_data_lo: u8`, `tile_data_hi: u8`.
- `mode: PpuMode` enum: `OamScan`, `Drawing`, `HBlank`, `VBlank` — so you
  always know which of the 4 phases ([section 2](#2-the-4-modes-and-what-happens-during-each-line))
  you're in.

Behavior to add, driven by your existing `dots` counter:

1. When entering Mode 3 (dots == 80) for a line: reset `lcd_x = 0`,
   `fetcher_tile_col = 0`, `fetcher_step = GetTile`, clear `bg_fifo`.
2. While in Mode 3: advance the fetcher state machine
   ([section 5.1](#51-the-fetchers-5-steps)) and attempt to output one
   pixel ([section 5.2](#52-every-dot-independently-try-to-output-a-pixel)),
   each dot.
3. When `lcd_x` reaches 160: switch to Mode 0 for the rest of the line.
4. If LCDC bit 0 is 0 (background disabled): skip all of the above, just
   write shade 0 (white) for the entire line.

> **Implementation note:** your existing `PPU::tick()` advances `dots` by
> 4 at a time (one CPU instruction step's worth), not by 1 dot at a time.
> The simplest fix: inside `tick()`, run the "one dot's worth of fetcher +
> pixel output logic" in a small loop, 4 times, instead of rewriting your
> whole timing loop to be called once per dot.

## 7. What this guide deliberately does NOT cover

To keep this approachable, the following real behaviors are skipped. They
don't break anything you build here — they're additions for later:

- **Window layer**: not handled at all yet. Good news: it reuses this
  exact same fetcher and FIFO, just with a trigger condition that swaps
  which tile map/row is being read. Not a rewrite, an addition.
- **Sprites (OBJs)**: not handled at all yet. This one IS a separate
  subsystem (its own FIFO, its own OAM-scanning logic, and pixel-mixing
  rules) — more work, added later on top of this.
- **Variable-length Mode 3**: real hardware's Mode 3 can take 172 to 289
  dots depending on scrolling/sprites/window. We use a fixed 172. See
  <https://gbdev.io/pandocs/Rendering.html#mode-3-length> for the real
  rules, when you're ready for them.
- **STAT interrupts** and **OAM DMA timing restrictions**: not covered
  here.

## 8. Summary cheat-sheet

| Concept | Where | Reference |
|---|---|---|
| Frame = 154 lines x 456 dots | — | Rendering.html |
| Line = Mode 2 (80) + Mode 3 (172, simplified) + Mode 0 (rest) | — | Rendering.html#ppu-modes |
| Tile map (which tile goes where) | `$9800`/`$9C00`, picked by LCDC bit 3 | Tile_Maps.html |
| Tile data (actual pixel shapes) | `$8000`/`$9000` base, picked by LCDC bit 4 | Tile_Data.html |
| 2bpp pixel decode formula | reuse from `vram_registers.rs` | Tile_Data.html |
| Scrolling | SCX (`$FF43`), SCY (`$FF42`), wraps mod 256 | Scrolling.html |
| Palette | BGP (`$FF47`), 2 bits per color index | Palettes.html |
| Real per-pixel mechanism | Fetcher (5 steps) + FIFO | pixel_fifo.html |
