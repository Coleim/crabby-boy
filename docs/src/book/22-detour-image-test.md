# 22. A Detour That Didn't Stick

This chapter is a little different: it's about a design that was tried
(`d7c25fd`) and then abandoned just one commit later (Chapter 23). It's
worth covering anyway — recognizing a dead end quickly is a normal, even
healthy part of building something like this, and the attempt still
taught useful things that carried forward.

## The idea: render the Game Boy screen as an image, in the terminal

```rust
// src/display_interface.rs (short-lived)
pub struct DisplayInterface {
    pub running: bool,
    image_state: RefCell<StatefulProtocol>,
}

impl DisplayInterface {
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio().expect("impossible de détecter le terminal");
        let img = generate_buffer();
        let protocol = picker.new_resize_protocol(img);
        DisplayInterface { running: true, image_state: RefCell::new(protocol) }
    }
}
```

The crates here — [`ratatui`](https://ratatui.rs/) for building terminal
UIs, and [`ratatui-image`](https://docs.rs/ratatui-image/) specifically
for rendering real bitmap images inside a terminal (many modern
terminals support protocols like Sixel or Kitty's image protocol that
make this possible) — turned out to be exactly the right choice. The
160×144 placeholder gradient image here is a stand-in for what will
eventually be real PPU pixel output (the companion
[PPU Background Rendering Guide](../ppu-background.md) is what finally
produces real content for a widget just like this one).

## A small but interesting Rust pattern: `RefCell` for a `&self` render method

```rust
impl Widget for &DisplayInterface {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image_widget = StatefulImage::default();
        let mut protocol = self.image_state.borrow_mut();
        image_widget.render(main, buf, &mut *protocol);
    }
}
```

`ratatui`'s `Widget::render` only gives you `&self` (an immutable
reference), but `ratatui-image`'s stateful image widget needs to mutate
its internal protocol state every time it draws (e.g. to cache an
encoded version of the image). `RefCell` is Rust's way of allowing
*controlled, checked-at-runtime* mutation through a shared reference —
exactly the tool for this mismatch. This detail survives into the final
architecture even though the surrounding struct doesn't.

## Why this specific shape got abandoned

The very next commit (`a3ec152`, Chapter 23) rewrites this entirely —
not because `ratatui`/`ratatui-image` were the wrong tools (they weren't;
they stick around), but because of a timing mismatch this design didn't
yet account for: `DisplayInterface::update` expected to be driven once
per emulated step, with no clear separation between "how fast the CPU
emulation runs" and "how fast the screen redraws." A terminal UI
realistically redraws at a much lower rate than the CPU executes
instructions (dozens of times a second, at most, versus millions of CPU
steps per second) — baking the display directly into the same loop as
CPU execution, the way this first attempt implicitly did, doesn't scale
well once you actually want a responsive UI alongside accurate emulation
speed.

## What we have now

- Confirmation that `ratatui` + `ratatui-image` can render a real bitmap
  image inside a terminal — a genuinely useful finding, carried forward.
- A concrete example of over-coupling (display tightly bound to the
  emulation loop) to recognize and avoid in the next design.

## What's still missing (and about to be redesigned)

- No separation yet between "how fast the emulator ticks" and "how fast
  the screen redraws" — that's exactly what Chapter 23 introduces.
