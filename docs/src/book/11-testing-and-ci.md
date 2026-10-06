# 11. Test Infrastructure & Continuous Integration

With a real automated test harness in place (Chapter 9) and a growing
pile of test ROMs (Chapters 4, 5, 10), two things happen in parallel in
this part of the project's history: the test conventions get written
down properly, and a collaborator joins to wire up Continuous Integration
(CI) — automatically building and testing the project on every push.

## Writing down the test ROM conventions

Chapter 10 already introduced the two Blargg-style result conventions
(serial text vs. memory signature). This gets formalized into
`TEST_ROM_SPECS.MD`, summarized as one reference table:

| Test | Serial (SB/$81) | Memory ($A000+) | CGB required | End of test |
|---|:---:|:---:|:---:|---|
| `cpu_instrs` | ✅ | ❌ | No | Serial "Passed" or self-loop |
| `instr_timing` | ✅ | ❌ | No | Serial "Passed" or self-loop |
| `mem_timing` | ✅ | ❌ | No | Serial "Passed" or self-loop |
| `mem_timing-2` | ❌ | ✅ | No | Self-loop (`JP $`) |
| `dmg_sound` | ❌ | ✅ | No | Self-loop (`JP $`) |
| `halt_bug.gb` | ❌ | ✅ | No | Self-loop (`JR $`) |

Writing a table like this isn't busywork — it's what lets you add new
test ROMs later (sound tests, OAM tests) by just checking "which row does
this one match" instead of reverse-engineering its behavior from scratch
every time.

## Setting up CI: a GitHub Actions workflow

```yaml
# .github/workflows/rust.yml
name: Rust

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
```

This is about as simple as CI gets: on every push or pull request, spin
up a fresh Ubuntu machine, check out the code, build it, run `cargo
test`. Because Chapter 9 already turned test ROMs into real `#[test]`
functions, this "just works" — CI doesn't need to know anything about
Game Boys, cartridges, or opcodes; it only needs to know how to run a
Rust test suite.

## A real lesson: tests can be *too* slow for CI

The very first version of this workflow had its `cargo test` step
**commented out**, with a note that building alone was enough for now.
Why? Running the *entire* `cpu_instrs.gb` ROM (as opposed to its 11
smaller individual sub-tests) takes a very large number of emulated CPU
steps to finish — fine to run locally and wait, much less fine to run on
every single CI push if it meaningfully slows down feedback. The eventual
fix (`cea7aca`, "Activate all tests but too long all_cpu_instrs") was to
enable *all* the fast, individual test ROMs, while explicitly leaving the
one genuinely slow combined ROM test commented out:

```rust
// Too long
// cpu_instr_test!(test_all_cpu_instrs, "./tests/cpu_instrs.gb");
```

This is a reusable, general lesson for any project with a slow-but-
valuable test: don't disable your whole test suite because of one slow
member — isolate it, and keep the fast majority running on every commit.

## What we have now

- `TEST_ROM_SPECS.MD`, documenting exactly how to interpret each family
  of test ROM.
- A working GitHub Actions pipeline, building and testing on every push
  and pull request.
- A deliberate, documented exception for the one test ROM too slow to
  run on every CI push.

## What's still missing

- No PPU-specific test ROMs yet (like `dmg-acid2`, a well-known PPU
  rendering correctness test) — there's no real PPU rendering to test
  against yet. That starts in the very next chapter.
- No sound test ROMs passing yet either (`dmg_sound` is documented, not
  yet runnable) — that's Part VI.
