# 0. What's Actually Inside a Game Boy

Before writing a single line of Rust, let's build a mental map of the
machine we're about to simulate. You don't need to memorize this chapter —
come back to it whenever a later chapter mentions a term you don't
remember.

## What does "emulating" a Game Boy actually mean?

A real Game Boy is a small circuit board with a few specialized chips on
it. Each chip has one job:

- A **CPU** (the "brain") that executes a game's code, instruction by
  instruction.
- A **PPU** ("Picture Processing Unit") that turns data in memory into the
  pixels you see on the screen.
- An **APU** ("Audio Processing Unit") that generates sound.
- Some **memory** chips (ROM on the cartridge, RAM inside the console and
  sometimes on the cartridge too).
- Some **buttons** (the joypad).

"Emulating" the Game Boy means writing a program that behaves exactly like
this hardware would, instruction by instruction, cycle by cycle — so that
an unmodified original game, which has no idea it's not running on real
silicon, still works correctly. We're not reimplementing *games*; we're
reimplementing the *machine* they run on.

Everything in this book is building, piece by piece, a software model of
each chip above, and wiring them together exactly like the real circuit
board wires them together.

## The CPU: a Sharp LR35902

The Game Boy's CPU is a custom chip similar to two older, well known CPUs
mashed together (Z80 and 8080, if those names mean anything to you — if
not, don't worry about it). What matters for us:

- It has a handful of **registers**: small, named storage slots that hold
  one byte (or, combined in pairs, two bytes) each. Registers are named
  `A`, `B`, `C`, `D`, `E`, `H`, `L` (one byte each), plus a special
  `F` register that holds **flags** (more on those when we get there), a
  16-bit **stack pointer** (`SP`), and a 16-bit **program counter** (`PC`)
  that always points at the next instruction to execute.
- It runs in a loop: **fetch** the byte at `PC`, **decode** which
  instruction that byte means, **execute** it, repeat forever. This is
  called the fetch-decode-execute cycle, and it's the heartbeat of every
  CPU, not just the Game Boy's.
- Every instruction takes a fixed, known number of **cycles** (think of a
  cycle as "one tick of the hardware clock") to run. Getting this timing
  right matters a lot — more on why in Part IV.

## Memory: one giant array, divided into zones

The Game Boy CPU can address 65,536 different memory locations (addresses
`0x0000` to `0xFFFF` — that's what 16 bits gets you: 2^16 = 65536). This
whole range is called the **memory map**, and different chunks of it mean
completely different things:

| Address range | What lives there |
|---|---|
| `0x0000`–`0x3FFF` | ROM, bank 0 (fixed, from the cartridge) |
| `0x4000`–`0x7FFF` | ROM, switchable bank (from the cartridge, see Chapter 8) |
| `0x8000`–`0x9FFF` | VRAM — video memory, read by the PPU |
| `0xA000`–`0xBFFF` | External RAM (on the cartridge, if present) |
| `0xC000`–`0xDFFF` | Work RAM — the console's own general-purpose RAM |
| `0xFE00`–`0xFE9F` | OAM — sprite attribute memory, also read by the PPU |
| `0xFF00`–`0xFF7F` | I/O registers — this is how the CPU talks to the PPU, APU, timer, joypad, etc. |
| `0xFF80`–`0xFFFE` | High RAM — a small extra scratch area |
| `0xFFFF` | One single byte: the interrupt enable register |

That's a lot of names you've never heard before — don't worry, each one
gets its own chapter. The important idea for now: **reading or writing a
"memory address" might not touch RAM at all** — depending on the address,
it might instead be reading a cartridge ROM byte, or writing a command to
the sound chip, or flipping a bit that controls the screen. In our code,
this routing logic lives in something we'll call the **Bus** (Chapter 6).

## I/O registers: the CPU's remote control for everything else

Many addresses in `0xFF00`–`0xFF7F` aren't memory at all — they're more
like labeled buttons and dials the CPU can read from or write to, to
control or query every other chip. For example (you'll meet all of these
properly later):

- `0xFF00` — joypad button state.
- `0xFF04`–`0xFF07` — the timer.
- `0xFF0F` and `0xFFFF` — interrupt flags (which events are pending /
  allowed to interrupt the CPU).
- `0xFF10`–`0xFF26` — sound channels.
- `0xFF40`–`0xFF4B` — PPU control (what to draw and how).

## The PPU: drawing one line at a time

The screen is 160×144 pixels. The PPU doesn't draw the whole image at
once — it draws it one horizontal line at a time, repeatedly, 60 times a
second, reading tile data from VRAM and sprite data from OAM. We'll get a
first, very minimal taste of this in Part V, and a much deeper, real
implementation is being built in the companion
[PPU Background Rendering Guide](../ppu-background.md) right after this
book's last chapter.

## The APU: four sound channels

The APU has 4 independent sound-generating "channels": two square waves,
one customizable waveform, and one noise generator, all mixed together
into the final audio you hear. Part VI covers each one.

## Interrupts: hardware tapping the CPU on the shoulder

Sometimes a chip needs to tell the CPU "something happened, pause what
you're doing and go handle it" — a new frame finished drawing, a timer
ran out, a button was pressed. This mechanism is called an **interrupt**,
and it's central enough that both the PPU stub (Part V) and the timer
(Part IV) only become truly useful once interrupts work. We cover this
properly in Chapter 14.

## A note on "DMG"

You'll see the abbreviation **DMG** in file names, register descriptions,
and test ROMs throughout this book and the wider Game Boy dev community.
It stands for "Dot Matrix Game" — Nintendo's internal codename for the
original 1989 Game Boy hardware (as opposed to later color/advance
models). This project targets DMG only.

## What's next

With this map in hand, Chapter 1 starts the actual project: reading a ROM
file into memory and standing up the first few Rust files.
