# 17. Channel 2 — The Simpler Square Wave

Channel 2 (`a10eb95`) is, by design, almost nothing new to build: it's
"channel 1 minus the sweep feature." This chapter is short on purpose —
the real lesson is in the design decision, not in new code.

## Reusing the exact same `Channel` struct

```rust
// src/audio/apu.rs
pub struct APU {
    // ...
    channel1: Channel,
    channel2: Channel,
    // ...
}
```

Both fields have the same type. Channel 2 simply never calls
`write_sweep`, never has anything clock its (unused) sweep fields, and
its own register-writing methods (`write_nr1`/`write_nr2`/etc., shared
with channel 1 via the same `impl Channel` block from Chapter 16) just
happen to be all it needs.

## Where the real differences live: in `IOBridge`'s address routing

The actual difference between the two channels isn't in any channel
logic at all — it's purely in which I/O addresses route to which
channel instance:

```rust
// channel 1 registers: NR10-NR14, conventionally 0xFF10-0xFF14
// channel 2 registers: NR21-NR24, conventionally 0xFF16-0xFF19 (no NR20 sweep register — skipped on purpose)
```

This is worth noticing as a general pattern: sometimes the most faithful
way to represent "feature X doesn't exist on this variant" isn't a
conditional flag somewhere in shared logic — it's simply *never wiring up
the register address that would control it* in the first place. Channel
2's hardware has no sweep register at all; our code mirrors that by
having no code path that would ever call `write_sweep` on channel 2's
`Channel` instance, rather than, say, a boolean flag like
`has_sweep: bool` that every method would need to check.

## What we have now

- Both square-wave channels (1 and 2) producing a duty-cycle waveform
  with envelope and length-counter support, sharing one implementation.
- A clean illustration of how much of the Game Boy's apparent complexity
  (4 "different" channels) is really a small number of shared building
  blocks (duty-cycle generator, envelope, length counter, sweep)
  recombined in slightly different ways per channel.

## What's still missing

- Channel 3 (arbitrary waveform, not duty-cycle-based) and channel 4
  (noise, not periodic at all) genuinely do need their own distinct
  implementations — covered next.
