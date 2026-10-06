# 26. Visualizing VRAM — Your First Look at Tile Data

This chapter (`cc4a9d1`) is the natural bridge into the book's final
part: it's the first time this project actually **decodes and displays
real tile graphics data** — exactly the pixel-decoding math the companion
[PPU Background Rendering Guide](../ppu-background.md) needs, built here
first as a standalone debug view, with no PPU timing/scanline concerns
attached yet.

## The approach: decode every tile in VRAM into one big image

```rust
// src/display/vram_registers.rs
pub fn render(frame: &mut Frame, area: Rect, vram: &[u8]) {
    let dyn_img = generate_buffer(vram);
    // ... same ratatui-image rendering pattern as Chapters 22/25
}

const COLORS: [Srgb; 4] = [
    Srgb::new(1.0, 1.0, 1.0),
    Srgb::new(0.83, 0.83, 0.83),
    Srgb::new(0.5, 0.5, 0.5),
    Srgb::new(0.0, 0.0, 0.0),
];

fn generate_buffer(vram: &[u8]) -> DynamicImage {
    const TILES_PER_ROW: usize = 32;
    const BYTES_PER_TILE: usize = 16;
    let number_of_tiles = vram.len() / BYTES_PER_TILE;
    let tile_rows = number_of_tiles.div_ceil(TILES_PER_ROW);
    let width_px = TILES_PER_ROW * 8;
    let height_px = tile_rows * 8;

    let mut raw_pixels: Vec<u8> = Vec::with_capacity(width_px * height_px * 4);
    for y in 0..height_px {
        for x in 0..width_px {
            let tile_idx = (y / 8) * TILES_PER_ROW + (x / 8);
            let offset = tile_idx * BYTES_PER_TILE;
            let row = y % 8;
            let lo = vram.get(offset + row * 2).copied().unwrap_or(0);
            let hi = vram.get(offset + row * 2 + 1).copied().unwrap_or(0);

            let pixel_idx_x = 7 - (x % 8);
            let color_idx: u8 = (((hi >> pixel_idx_x) & 1) << 1) | ((lo >> pixel_idx_x) & 1);
            let color: Srgb<u8> = COLORS[color_idx as usize].into_format();

            raw_pixels.push(color.red);
            raw_pixels.push(color.green);
            raw_pixels.push(color.blue);
            raw_pixels.push(255);
        }
    }
    DynamicImage::ImageRgba8(RgbaImage::from_raw(width_px as u32, height_px as u32, raw_pixels).unwrap())
}
```

If this 2-bits-per-pixel decoding formula
(`(((hi >> bit) & 1) << 1) | ((lo >> bit) & 1)`) looks familiar, it's
because it's the exact same tile-data format explained conceptually back
in Chapter 0's memory map and (indirectly) used since Chapter 2's byte
interpretation discussion — this is simply the first chapter that
actually *implements* reading it. Every 16-byte chunk of VRAM is treated
as one 8×8 tile; `TILES_PER_ROW = 32` lays all of them out side by side,
32 per row, into one big image — which is not itself a meaningful "image"
a game ever displays as-is (it ignores tile maps, scrolling, and
palettes entirely), but is an extremely useful *raw* view: it shows every
distinct 8×8 graphic currently sitting in VRAM, regardless of whether or
how the PPU would currently choose to arrange them on screen.

## A fixed, hardcoded palette — and why that's fine, here

```rust
const COLORS: [Srgb; 4] = [
    Srgb::new(1.0, 1.0, 1.0), // color index 0 → white
    Srgb::new(0.83, 0.83, 0.83),
    Srgb::new(0.5, 0.5, 0.5),
    Srgb::new(0.0, 0.0, 0.0), // color index 3 → black
];
```

Chapter 0 mentioned that the actual on-screen shade for a given 2-bit
color index depends on the `BGP` palette register (Chapter 7), not just
the raw index. This debug viewer intentionally skips that step and maps
index → shade directly, 1-to-1 — appropriate for a tool whose purpose is
"show me the raw tile *shapes* regardless of any specific game's current
palette," rather than "show me exactly what the screen would currently
look like." Knowing when a simplification is appropriate for the tool
you're building (a raw debug view) versus when it would be wrong (an
accurate emulated display) is itself a useful skill.

## What we have now

- Accurate 2-bits-per-pixel tile decoding, reusable wherever real tile
  graphics need to be turned into pixels.
- A terminal-rendered view of every tile currently stored in VRAM, using
  the same `ratatui-image` approach established in Chapters 22 and 25.
- Every piece this project has built so far — CPU, bus, timer,
  interrupts, APU, joypad stub, terminal UI, and now real tile decoding —
  sitting alongside a PPU (Chapter 12) that still only tracks scanline
  timing.

## What's still missing

- No tile *maps* are read (Chapter 0/the companion guide's "tile map"
  concept) — this shows every tile that exists in VRAM, not which ones a
  game has actually chosen to place where on its background.
- No scrolling (`SCX`/`SCY`), no palette (`BGP`), no window, no sprites —
  all explicitly out of scope for this raw debug tool, and all picked up
  properly in the next, final chapter's forward-look.
