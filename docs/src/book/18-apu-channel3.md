# 18. Channel 3 — Custom Wave

Channel 3 (`16b3864`) is the first channel that genuinely needs its own
struct: instead of a fixed 4-shape duty cycle (Chapter 16), it plays back
an arbitrary, game-supplied 32-sample waveform, on a loop, at a
controllable pitch and volume.

## `WaveChannel`

```rust
// src/audio/wave_channel.rs
#[derive(Default)]
pub struct WaveChannel {
    pub dac_enabled: bool,
    pub enabled: bool,
    pub initial_len_timer: u16,
    pub len_timer: u16,
    pub length_enabled: bool,
    pub sample_countdown: u16,
    pub volume_level: u8,
    pub period: u16,
    pub wave_ram: [u8; 16],
    pub wave_index: u8, // 0..31 (32 nibbles total)
    pub sample_buffer: u8,
    pub divider_phase: bool,
    pub wave_form_just_read: bool,
    pub wave_access_window: u8,
}
```

## Wave RAM: 16 bytes holding 32 samples

`wave_ram` is 16 bytes, but the waveform itself has 32 samples
(`wave_index` ranges 0..31) — each byte packs **two 4-bit samples**
(a "nibble" each), the same "pack two small values into one byte" idea
you've now seen several times (2-bit flags in `F`, Chapter 3; 2-bit color
indices in VRAM tiles, in the companion PPU guide). A game writes
whatever 32-sample waveform it wants into this region (`0xFF30`-`0xFF3F`
in the real memory map) before playing channel 3 — this is what makes
channel 3 sound different from game to game, unlike channels 1/2/4 whose
sound character is fixed by hardware.

## A dedicated enable/disable flag: the DAC

```rust
pub fn write_nr0(&mut self, val: u8) {
    self.dac_enabled = val & 0b1000_0000 != 0;
    if !self.dac_enabled {
        self.enabled = false;
    }
}
```

This introduces a distinction worth understanding once, since it recurs
for every channel (noticeable again in Chapter 19's noise channel): a
channel has both an **enabled** flag (is it currently actively playing,
e.g. has its length counter not yet run out) and a separate **DAC
enabled** concept (is its internal digital-to-analog converter even
switched on at all). Turning the DAC off immediately silences the
channel regardless of anything else — it's a more fundamental "off
switch" than the length counter or envelope.

## Register layout: period split across two registers

```rust
pub fn write_nr3(&mut self, val: u8) {
    self.period = (self.period & 0b111_0000_0000) | val as u16;
}

pub fn write_nr4(&mut self, val: u8, length_clock_on_write: bool) {
    // ...
    self.period = (self.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);
    self.length_enabled = val & 0b0100_0000 != 0;
    // ...
}
```

The channel's pitch ("period") is an 11-bit value, too wide for one
8-bit register, so it's split: `NR3` holds the low 8 bits, `NR4` holds
the high 3 bits (plus unrelated flags like `length_enabled` packed into
its other bits). Each write combines the new byte with a bitmask
preserving the *other* half of the period that this particular register
doesn't own (`self.period & 0b111_0000_0000` keeps the high bits
unchanged while replacing the low ones, and vice versa). This "split one
logical value across two addressable registers, with masks to avoid
clobbering the other half" pattern is common across Game Boy I/O
registers — you'll want to recognize it on sight.

## The DMG length-counter quirk, written down in code

```rust
// DMG quirk: enabling length can immediately clock it depending on frame
// phase.
if !was_length_enabled && self.length_enabled && length_clock_on_write && self.len_timer > 0 {
    self.len_timer = self.len_timer.saturating_sub(1);
    // ...
}
```

Another example of the "looks like a bug, is actually documented hardware
behavior" theme from Chapter 16: on original DMG hardware, flipping the
length-enable bit on at exactly the wrong moment in the frame sequencer's
cycle causes one extra, immediate length-counter decrement, as a side
effect of how the length counter's clock and the enable-bit check happen
to interact in the real circuit. `length_clock_on_write` is how the APU
communicates "are we currently at one of those exact moments" into this
method.

## What we have now

- Channel 3 playing back an arbitrary, game-provided waveform at
  controllable pitch/volume.
- A clearer understanding of DAC-enabled vs. channel-enabled as two
  separate concepts.
- The split-register and quirky-length-counter patterns, both of which
  reappear in channel 4.

## What's still missing

- Channel 4 (noise) still to come, next chapter.
- Nothing here is yet verified against Blargg's `dmg_sound` test suite —
  that's Chapter 20, after all 4 channels exist.
