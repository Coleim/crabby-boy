# 25. Displaying CPU Registers and Cartridge Info

With a solid display architecture in place (Chapter 23), this chapter
(`6846f97`) starts actually using it for something genuinely useful: a
live, human-readable debug view straight out of the CPU's internal state
— invaluable for understanding what your own emulator is doing while it
runs, not just whether it passes tests.

## Each widget is just a function: `(Frame, Rect, data) -> ()`

```rust
// src/display/registers.rs
pub fn render(frame: &mut Frame, area: Rect, cpu: &CPU) {
    let block = Block::bordered()
        .title(Line::from("CPU"))
        .style(Style::new().light_magenta());

    let rows = vec![
        Row::new(vec![
            Cell::from(Line::from(vec!["AF".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.get_af()).bold().white()])),
            Cell::from(Line::from(vec!["BC".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.get_bc()).bold().white()])),
            Cell::from(Line::from(vec!["PC".bold().gray()])),
            Cell::from(Line::from(vec![format!("{:04X}", cpu.pc).bold().white()])),
        ]),
        // ... DE / HL / SP row
    ];

    let table = Table::new(rows, /* column widths */).block(block);
    frame.render_widget(table, area);
}
```

This is the simplest possible shape a `ratatui` widget function can
take: given a `Frame` to draw into, a screen region (`Rect`), and a piece
of emulator state to read from, build and render some UI. No struct, no
trait implementation needed for this part — `ratatui_display.rs` (from
Chapter 23) just calls `registers::render(frame, area, &crabby.cpu)`
directly inside its own `draw`. This matters because it's a reminder:
`get_af()`/`get_bc()`/`get_hl()` (the register-pair getters from Chapter
4) are now paying off somewhere completely unrelated to opcode decoding —
a well-named, well-tested small building block tends to get reused in
places you didn't originally plan for.

## Reusing the cartridge header for a second time

```rust
// src/display/cartridge_header.rs (sketch, same pattern as registers.rs)
pub fn render(frame: &mut Frame, area: Rect, header: &CartdrigeHeader) {
    // title, cartridge type, ROM/RAM size, etc., each as a labeled row
}
```

`CartdrigeHeader` (Chapter 2) already had a `print()` method for raw
console debugging since its very first version — now its parsed fields
get a second, nicer presentation: a bordered panel in the terminal UI,
instead of (or alongside) plain `println!` output. The underlying data
and parsing logic doesn't change at all; only how it's displayed does —
a good example of keeping "what the data is" separate from "how it's
shown," so the same source of truth can serve multiple presentations.

## A first (still placeholder) game screen

```rust
// src/display/game.rs
const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

pub fn render(frame: &mut Frame, area: Rect) {
    // TODO: Replace with real PPU buffer
    let dyn_img = generate_buffer(); // still a gradient test pattern
    // ... same ratatui-image rendering approach as Chapter 22's detour
}
```

This is the direct continuation of Chapter 22's abandoned
`DisplayInterface` idea, now properly slotted into the real architecture:
a 160×144 image widget, sized exactly to match the real Game Boy screen
resolution (Chapter 0/12), currently fed a placeholder gradient instead
of real pixels — explicitly marked with a `TODO` as exactly that. This is
the socket the companion
[PPU Background Rendering Guide](../ppu-background.md) eventually plugs
real rendered frames into.

## What we have now

- A live CPU register view and cartridge header view in the terminal UI.
- A reusable small-widget-function pattern for adding more debug views
  later (this is exactly the same pattern Chapter 26's VRAM viewer
  follows next).
- A correctly-sized placeholder game screen, ready for real pixels.

## What's still missing

- The game screen still shows a gradient, not real Game Boy output.
- No VRAM visualization yet — that's next, and it's the last stepping
  stone before this book's closing chapter.
