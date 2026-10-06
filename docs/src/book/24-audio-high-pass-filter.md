# 24. A High-Pass Filter for Cleaner Audio

With all 4 channels passing Blargg's tests (Chapter 20), the raw mixed
output can still have an unwanted side effect: a slowly drifting DC
offset (the waveform's average value isn't centered on zero), which
tends to sound like an audible hum or "thump" rather than clean audio.
This small commit (`58e899f`) fixes that with a classic piece of signal
processing: a **high-pass filter**.

## What a high-pass filter does, conceptually

A high-pass filter lets fast-changing signal content through, while
gradually "forgetting"/removing slow, near-constant drift. In audio
terms: it keeps the actual sound you want, while removing a DC offset or
very low-frequency rumble that shouldn't be there. This specific design
is a simple one-pole filter — only needing to remember the *previous*
input and output sample, not a whole history.

## The implementation

```rust
// src/audio/audio_output.rs
let mut hp_x1 = 0.0; // previous input sample
let mut hp_y1 = 0.0; // previous output sample
let hp_cutoff_hz = 20.0;
let dt = 1.0 / selected_sample_rate as f32;
let rc = 1.0 / (2.0 * std::f32::consts::PI * hp_cutoff_hz);
let hp_alpha = rc / (rc + dt);

// ... inside the audio callback, once per sample:
hp_y1 = hp_alpha * (hp_y1 + last_sample - hp_x1);
hp_x1 = last_sample;
for out in frame.iter_mut() {
    *out = hp_y1;
}
```

`hp_cutoff_hz = 20.0` sets the filter's "cutoff frequency" — roughly, the
threshold below which content gets attenuated. 20 Hz is right at the
bottom edge of human hearing, chosen specifically to remove DC drift and
sub-audible rumble while leaving every actually audible frequency
untouched. `rc` and `hp_alpha` come from the standard formula for this
kind of filter (a "first-order RC high-pass filter," if you want to look
up the general electronics theory) — the important takeaway for this
project isn't deriving the formula yourself, but recognizing *when* you
need one: whenever mixed audio output has an audible hum/thump that
test-ROM-style correctness checks (Chapter 20) wouldn't catch, since
those check logical register behavior, not the final analog-ish waveform
quality.

## Where this filter sits

Notice this lives in `audio_output.rs` — the layer that talks to `cpal`
and the real sound card (Chapter 15) — not inside the APU itself. The APU
stays focused on accurately emulating what real Game Boy hardware
computes; final-stage signal cleanup for *this particular emulator's*
output path is a separate, presentation-level concern, kept in its own
place.

## What we have now

- Cleaner audio output, with DC offset and sub-audible rumble filtered
  out before reaching real speakers.
- A clear separation between "accurate hardware emulation" (APU) and
  "output quality polish" (`audio_output.rs`) — worth keeping in mind as
  a general design principle.

## What's still missing

- This is a presentation-layer fix, not a hardware-accuracy one — it has
  no bearing on any of the `dmg_sound` test ROMs from Chapter 20, which
  continue to pass or fail independently of it.
