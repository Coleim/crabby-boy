# 23. Decoupling Emulation Speed from Display Refresh

This commit (`a3ec152`, "refactor emulator to be able to be ticked from
external process... useful for having a UI that framerate differs from
CPU") is the direct fix for the coupling problem identified at the end of
Chapter 22. It introduces the architecture the project still uses today.

## A `Display` trait: the UI is now a pluggable interface

```rust
// src/display/display.rs
pub trait Display {
    fn draw(&mut self, emulator: &CrabbyBoy);
    fn handle_events(&mut self);
    fn is_running(&self) -> bool;
}
```

Instead of one concrete struct owning both "run the emulator" and "draw
to the terminal" (Chapter 22's `DisplayInterface`), there's now a small
trait any display implementation can satisfy. `CrabbyBoy` (the emulator
itself) and whatever implements `Display` become two independent things,
only connected by `draw` being handed a read-only reference to the
emulator's current state.

## `CrabbyBoy::tick_for_duration`: run "about this many cycles," not "one step"

```rust
// src/crabby_boy.rs
const RUNTIME_STEPS_PER_SEC: f64 = 400_000.0;

pub fn tick_for_duration(&mut self, frame_delta: Duration) {
    let dt = frame_delta.as_secs_f64().clamp(0.001, 0.5);
    let number_of_steps = (RUNTIME_STEPS_PER_SEC * dt) as usize;
    for _ in 0..number_of_steps {
        self.tick();
    }
}
```

This is the key new idea. Instead of the main loop stepping the CPU
exactly once per iteration (as it has since Chapter 9), the caller now
says "however much real time just passed (`frame_delta`), run
*approximately* that much emulated time's worth of steps." The display
loop can redraw at whatever rate makes sense for a terminal (a handful of
times per second) while the emulator inside still advances at the right
overall speed, in a burst, between redraws.

## The main loop, now driven from `main.rs`, not `emulator.rs`

```rust
// src/main.rs
let mut crabby = CrabbyBoy::new(file_path)?;
let mut display = RatatuiDisplay::new();
let target_frame = Duration::from_micros(16_667); // ~60 FPS
let mut last_tick_instant = Instant::now();

while display.is_running() {
    let frame_start = Instant::now();
    let dt = frame_start.duration_since(last_tick_instant);
    last_tick_instant = frame_start;

    display.handle_events();
    crabby.tick_for_duration(dt);
    display.draw(&crabby);

    let frame_elapsed = frame_start.elapsed();
    if frame_elapsed < target_frame {
        std::thread::sleep(target_frame - frame_elapsed);
    }
}
```

Compare this to Chapter 9's original loop, which lived entirely inside
`CrabbyBoy::run` and never returned until the whole ROM finished (or a
test condition was met). Now, `main.rs` owns the outer loop, measures
real elapsed wall-clock time (`dt`) each iteration, and explicitly
paces itself to roughly 60 frames per second — sleeping out any leftover
time if an iteration finished early. This is the standard shape of a
real-time simulation loop: measure elapsed time, advance the simulation
by that much, render, repeat.

## `RatatuiDisplay`: the first concrete `Display` implementor

```rust
// src/display/ratatui_display.rs (sketch)
pub struct RatatuiDisplay {
    terminal: DefaultTerminal,
    running: bool,
    fps: FpsCounter,
}
```

A new `FpsCounter` (`src/display/fps_counter.rs`) tracks real measured
frame rate for on-screen debugging — handy once you're deliberately
pacing a loop like this, since it's easy to introduce subtle bugs that
make it run faster or slower than intended.

## A quick, honest follow-up fix

The very next commit, `8b32399` ("Fixing test after refactor. It's
cleaner now... Love it!"), exists because this refactor — like most
refactors touching a central loop — broke the existing automated test
suite (Chapter 9/11's `cpu_instr_test!` macro), which had been calling
the old `CrabbyBoy::run`/single-`tick` shape directly. Updating tests to
match a new architecture, right after introducing it, is completely
normal and worth normalizing rather than hiding.

## What we have now

- A clean `Display` trait, decoupling "how the emulator runs" from "how
  it's shown."
- Real-time-paced ticking (`tick_for_duration`), letting emulation speed
  and display refresh rate differ.
- The first working `RatatuiDisplay`, and an `FpsCounter` to keep it
  honest.

## What's still missing

- `RatatuiDisplay` at this point still draws almost nothing meaningful —
  the actual CPU register/header views (Chapter 25) and VRAM tile viewer
  (Chapter 26) are built on top of this foundation in the chapters ahead.
