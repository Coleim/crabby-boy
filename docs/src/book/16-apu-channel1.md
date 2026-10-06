# 16. Channel 1 — Square Wave with Sweep

Channel 1 is the first channel to get real behavior (`a8e1bcf`, `be5f728`,
`ff84e29`, `36dd81a`), and it's the most feature-complete of the two
square-wave channels: on top of the duty cycle and envelope every square
channel has, channel 1 alone also supports **frequency sweep** — a
smooth pitch slide, the classic "pew!" laser-sound-effect feature.

## The shared `Channel` struct

```rust
// src/audio/channel.rs
#[derive(Default)]
pub struct Channel {
    pub duty_cycle: u8,
    pub duty_pos: u8,
    pub length_timer: u8,
    pub initial_length_timer: u8,
    pub length_enabled: bool,
    pub env_timer: u8,
    pub env_dir: u8,  // 0 = down, 1 = up
    pub env_pace: u8, // 0..7, envelope speed
    pub period: u16,
    pub volume: u8,
    pub initial_volume: u8,
    pub freq_timer: u32,
    pub enabled: bool,
    // sweep (channel 1 only — channel 2 just never uses these fields)
    pub sweep_pace: u8,
    pub sweep_substraction: bool,
    pub sweep_step: u8,
    pub sweep_timer: u8,
    pub sweep_enabled: bool,
    pub sweep_shadow_period: u16,
    pub sweep_negate_used: bool,
}
```

Reusing one struct for both square channels (channel 2 simply never
triggers the sweep-related fields) avoids duplicating duty-cycle and
envelope logic — a reasonable tradeoff of "slightly unused fields on
channel 2" versus "two near-identical structs to keep in sync."

## Parsing the NRx registers

Each channel is controlled by a small handful of dedicated registers
(named `NR1x` for channel 1, `NR2x` for channel 2, etc. in Game Boy
documentation). Parsing them is mostly bit-field extraction, the same
skill from Chapter 4's `0xCB`-prefixed opcode decoding:

```rust
pub fn write_nr1(&mut self, val: u8) {
    self.duty_cycle = (val & 0b1100_0000) >> 6;
    self.initial_length_timer = 64 - (val & 0b0011_1111);
    self.length_timer = self.initial_length_timer;
}

pub fn write_nr2(&mut self, val: u8) {
    self.initial_volume = (val & 0b1111_0000) >> 4;
    self.env_dir = (val & 0b0000_1000) >> 3;
    self.env_pace = val & 0b0000_0111;
}

pub fn write_sweep(&mut self, val: u8) {
    self.sweep_pace = (val & 0b0111_0000) >> 4;
    let old_sub = self.sweep_substraction;
    self.sweep_substraction = val & 0b0000_1000 != 0;
    self.sweep_step = val & 0b0000_0111;

    // CH1 quirk: leaving "subtract" mode right after a subtract-mode
    // calculation was used silently disables the whole channel.
    if old_sub && !self.sweep_substraction && self.sweep_negate_used {
        self.enabled = false;
        self.sweep_enabled = false;
    }
}
```

That last quirk in `write_sweep` is a good example of something you'll
run into constantly writing a faithful emulator: documented, deliberate
hardware edge cases that look like bugs, verified against test ROMs
(Chapter 20) rather than derived from first principles. Nobody designs a
feature where "changing a setting back can silently turn off the whole
channel" on purpose from a product perspective — but real silicon does
exactly this, games may rely on it (intentionally or not), and an
accurate emulator has to reproduce it.

## Sweep: computing the next frequency

```rust
pub fn sweep_next_period_and_overflow(&self) -> (u16, bool) {
    let delta = self.sweep_shadow_period >> self.sweep_step;
    if self.sweep_substraction {
        (self.sweep_shadow_period.saturating_sub(delta), false)
    } else {
        let next = self.sweep_shadow_period as u32 + delta as u32;
        (next as u16, next > 0x7FF)
    }
}
```

Each sweep step (clocked by the frame sequencer from Chapter 15, at 128
Hz), the channel's frequency ("period," here) shifts up or down by a
fraction of its current value (`>> sweep_step` — a bigger shift value
means a smaller fractional change, hence a slower sweep). `0x7FF`
(2047) is the largest value the period register can hold; sweeping
past it means the frequency has gone out of representable range, which
silently disables the channel (`calculate_new_period(sweep) > 2047` in
the earlier design notes in `APU.MD.md`, implemented here as the
`overflow` boolean this function returns).

## What we have now

- Channel 1's register parsing: duty cycle, length, envelope, and sweep.
- Real sweep computation, including the overflow-disables-channel and
  mode-switch-disables-channel quirks.
- The shared `Channel` struct, ready to be reused as-is for channel 2.

## What's still missing

- Channel 2 itself isn't wired up yet (next chapter — it reuses
  everything built here).
- No audible verification yet beyond informal listening (`be5f728`,
  "BIP sound," was literally the first audible beep) — formal test-ROM
  validation is Chapter 20.
