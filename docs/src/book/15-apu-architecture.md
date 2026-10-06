# 15. APU Architecture & the `audio/` Module

With the CPU, bus, timer, and a first PPU all in reasonable shape, this
part of the project's history (`7d79a24` onward) turns to sound. Sound
and graphics are genuinely independent subsystems — this is why, in the
project's real history, APU work happens *after* the minimal PPU
(Chapter 12) rather than after a complete one: nothing about generating
audio depends on pixels being drawn.

## A dedicated `audio/` module

```text
src/
  audio/
    apu.rs           — the APU itself: 4 channels, mixing, frame sequencer
    audio_buffer.rs  — a ring buffer handing samples to the OS
    audio_output.rs  — talks to the actual sound card, via cpal
    channel.rs       — shared logic for channels 1 and 2 (square waves)
    wave_channel.rs  — channel 3 (custom waveform)
    noise_channel.rs — channel 4 (noise)
```

`hardware/apu.rs` (Chapter 6) moves out into its own top-level module,
with room for everything sound-related to live together, mirroring how
`cpu/`, `bus/`, and `hardware/` were split out earlier (Chapter 6).

## Four channels, one mixer

```rust
// src/audio/apu.rs
pub struct APU {
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    div_apu_counter: u16,
    frame_seq_step: u8,
    tick_counter: f64,
    tick_per_sample: f64,
    is_on: bool,
    channel1: Channel,     // square wave + sweep
    channel2: Channel,     // square wave
    channel3: WaveChannel, // custom waveform
    channel4: NoiseChannel,
    nr50: u8, // master volume
    nr51: u8, // stereo panning
}
```

Exactly matching Chapter 0's quick mention: 4 independent sound
generators running in parallel, combined down to a final signal. Channels
1 and 2 share one `Channel` struct (channel 2 is simply channel 1 without
the sweep feature) — reusing the same struct avoids duplicating duty-cycle
and envelope logic twice.

## The frame sequencer: one shared clock for all channels' "slow" features

Each channel has its own fast audio-generation clock (producing the
actual waveform), but several *slower* features — envelope (volume
fading), sweep (frequency sliding, channel 1 only), and length counters
(auto-muting a channel after a set duration) — are all driven off one
shared 512 Hz clock, derived from the main CPU clock:

```rust
pub fn tick(&mut self) {
    for _ in 0..4 {
        self.div_apu_counter += 1;
        if self.div_apu_counter == 8192 {
            self.div_apu_counter = 0;
            self.frame_seq_step = (self.frame_seq_step + 1) & 0x07;
            if self.is_on {
                match self.frame_seq_step {
                    0 | 2 | 4 | 6 => self.clock_length_all(), // 256 Hz
                    _ => {}
                }
                if self.frame_seq_step == 7 {
                    self.clock_envelope_all(); // 64 Hz
                }
                if self.frame_seq_step == 2 || self.frame_seq_step == 6 {
                    self.clock_sweep(); // 128 Hz
                }
            }
        }
        if !self.is_on { continue; }
        self.channel1.tick();
        self.channel2.tick();
        // ...
    }
}
```

`div_apu_counter` reaching 8192 CPU cycles is exactly 512 Hz (4,194,304
Hz ÷ 8192 = 512). `frame_seq_step` then cycles through 8 steps (0-7), and
different features are clocked on different steps — length counters
every other step (256 Hz), sweep every 4th step (128 Hz), envelope once
per full cycle (64 Hz). This single shared "frame sequencer" is exactly
how real Game Boy hardware times these features, and is worth
remembering as a named concept if you read further hardware
documentation.

## Duty cycles: what makes a "square wave" have a shape

```rust
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 00 → 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 01 → 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 10 → 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 11 → 75%
];
```

A "square wave" isn't just one fixed shape — it's a repeating pattern of
high/low values, and *what fraction of each cycle is "high"* (the
**duty cycle**) changes its timbre, even at the same pitch. These 4 rows
are the 4 fixed patterns the real hardware supports, each one simply a
fixed sequence of 8 bits repeated over and over at the channel's current
frequency. Channel 1 and 2 (Chapter 16/17) both use this table; channel 3
(Chapter 18) instead reads an arbitrary, game-supplied waveform; channel 4
(Chapter 19) uses pseudo-random noise instead of any repeating shape at
all.

## Getting samples out to real speakers: `cpal`

```rust
// src/audio/audio_output.rs
pub struct AudioOutput {
    _stream: cpal::Stream, // must stay alive, or audio stops
}
```

This is the one place in the whole project, so far, that depends on an
external crate for something OS-specific: actually talking to the sound
card. [`cpal`](https://docs.rs/cpal/latest/cpal/) is a cross-platform
audio I/O crate — it finds an output device, picks a supported sample
format/rate, and gives you a callback that gets asked for fresh samples
whenever the OS needs more. The APU itself has no idea `cpal` exists: it
just writes finished samples into an `AudioBuffer` (a thread-safe ring
buffer), and `AudioOutput`'s callback reads from that same buffer
whenever the sound card asks for more data. This separation matters: the
APU runs in lockstep with CPU emulation speed, while the sound card
demands samples on its own schedule — the ring buffer is what lets those
two different timing worlds coexist safely across threads
(`Arc<Mutex<AudioBuffer>>`).

## What we have now

- A dedicated `audio/` module, cleanly separated from CPU/bus/hardware.
- The APU's frame sequencer, correctly timing envelope/sweep/length
  features at their real hardware frequencies.
- A first real connection to actual sound hardware via `cpal`.

## What's still missing

- No individual channel is fully implemented yet — this chapter is the
  shared scaffolding; Chapters 16-19 cover each channel in turn.
- No actual sample generation/mixing happening yet in a way you could
  meaningfully listen to.
