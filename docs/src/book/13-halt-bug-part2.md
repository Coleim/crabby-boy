# 13. The Halt Bug, Part 2

Chapter 5 implemented a first attempt at the HALT bug, with its own
dedicated test ROM, but flagged it as "not fully correct yet." This
chapter (`9e46890`) is where `halt_bug.gb` actually gets added to the
automated test suite and starts passing — and the real fix is a
wonderfully humbling reminder of how deep tiny bugs can hide.

## The actual bug: a missing zero

```diff
-            0xFF0F => self.interrupt_flag | 0b1110_000,
+            0xFF0F => self.interrupt_flag | 0b1110_0000,
```

That's it. `0b1110_000` is a **7-bit** literal (`0b1110000` = `0x70`);
the intended value `0b1110_0000` is the correct **8-bit** one (`0xE0`).
One missing digit. Chapter 12 explained that `IF`'s top 3 bits always
read back as `1` on real hardware — this line is exactly that masking —
and a single missing `0` meant bit 7 of `IF` was silently read as `0`
instead of `1` whenever the actual flag bits underneath happened to leave
it unset. `halt_bug.gb` precisely checks register values like this one,
byte for byte, which is exactly why it caught something that running
`cpu_instrs.gb` and `mem_timing.gb` never surfaced.

This is a genuinely useful lesson, maybe the most useful one in this
whole book: **the "halt bug" test failure wasn't actually about HALT
logic being wrong at all** — the HALT bug *implementation* from Chapter 5
was fine. The bug was one bit-width typo in a completely different
register, several chapters earlier, that only a very specific, narrowly
targeted test ROM happened to expose. This is exactly why dedicated,
narrow test ROMs (as opposed to only big general ones) earn their keep —
and exactly why, when a test fails, the bug is often not where the test's
name suggests you should look first.

## Two smaller correctness fixes alongside it

```rust
0xFEA0..=0xFEFF => {
    println!("Not Usable ... Addr: {:02x}", addr);
}
```

Writes to the `0xFEA0`–`0xFEFF` range (explicitly unusable memory, per
the map from Chapter 0) were already handled on the *read* side but not
the *write* side — now both log the same warning instead of falling
through to a generic catch-all.

```rust
cpu_instr_test!(halt_bug, "./tests/halt_bug.gb");
```

And `halt_bug.gb` finally joins the growing list of automated tests from
Chapter 9/11, now that it actually passes.

## What we have now

- A correct `IF` register read mask, and by extension, a `halt_bug.gb`
  test that passes.
- `halt_bug.gb` running as part of the automated CI suite from Chapter
  11.
- A completed write-side handler for the unusable OAM-adjacent memory
  range.

## What's still missing

- Nothing conceptually new about HALT itself remains — but this chapter
  is a good moment to remember: passing test ROMs builds *confidence*,
  not *proof*. Keep an eye out, in your own project, for "this test
  failure's name doesn't match where the actual bug turned out to live."
- Interrupt *dispatch* — actually jumping to an interrupt vector when one
  fires — still isn't implemented. That's next, in Chapter 14.
