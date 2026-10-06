# 21. The Joypad

This commit (`e128e88`, titled "Joypad all button released") is a good
example of a very common emulator-development move: fixing a
compatibility problem with the *simplest possible* correct-enough value,
rather than building the full feature right away.

## The problem this solves

```rust
pub fn read(&self) -> u8 {
    0xF // was: self.p1
}
```

Recall from Chapter 0/7 that the joypad register (`P1`, `0xFF00`) uses
**active-low** logic: a bit reading `0` means "this button is pressed,"
and `1` means "not pressed." Before this change, `read()` simply echoed
back whatever was last *written* to the register (Chapter 7's original
stub) — which is wrong in a way that matters: many games, during boot or
input-polling loops, write a "which button group do you want" selector
value and then immediately read back the result, expecting to see "no
buttons pressed" (all relevant bits `1`) if nothing is held down. Echoing
the write instead could make the game think buttons were being held that
weren't, confusing boot sequences or input logic. Hardcoding `0xF` (all 4
relevant bits set to `1`) says, unconditionally, "nothing is pressed,
ever" — not yet a real joypad, but enough to stop this class of bug.

## Why "all buttons released" is a perfectly good stepping stone

This is the same bootstrap-with-a-placeholder idea from Chapters 6 and 7
(hardcoded LCDC/STAT/LY values before a real PPU existed): a
*conservative*, always-safe placeholder lets dependent code (here, any
game's input-polling logic) proceed correctly for the common case ("is
anything pressed right now? No.") without yet investing in the full
feature (real keyboard-to-button mapping, the actual button-group-select
protocol from Chapter 0). As of this point in the project, that's exactly
where things stand — and it's still true in the current codebase: no
keyboard key is mapped to any Game Boy button yet. The terminal UI's
`handle_events` (Chapter 23) only recognizes one key, `q`, to quit:

```rust
// src/display/ratatui_display.rs
fn handle_events(&mut self) {
    if let Ok(Event::Key(key_event)) = event::read() {
        match key_event.code {
            KeyCode::Char('q') => self.running = false,
            _ => {}
        }
    }
}
```

## A small, unrelated fix riding along: wave RAM's address range

```diff
-            0xFF10..=0xFF26 => self.audio.read(addr),
+            0xFF10..=0xFF3F => self.audio.read(addr),
```

Channel 3's wave RAM (Chapter 18) actually lives at `0xFF30`-`0xFF3F`,
just past where the "normal" sound control registers end (`0xFF26`). This
one-line widening of the matched range is what made that whole address
block reachable at all — another example (like Chapter 13's missing
zero) of a small, easy-to-miss range boundary mattering a lot in
practice.

## What we have now

- A joypad register that reliably reports "nothing pressed" — enough for
  games whose boot/input logic only needs that guarantee to proceed
  correctly.
- Wave RAM's full address range finally reachable.

## What's still missing

- No real button-to-key mapping exists yet, anywhere in the project —
  this is an explicitly open piece of future work, not something this
  book's commit history has resolved yet.
- No Joypad interrupt (`IF` bit 4, mentioned since Chapter 14) is ever
  set, since nothing currently detects a real button press to trigger
  it.
