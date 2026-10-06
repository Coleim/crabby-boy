# 2. Parsing the Cartridge Header

Every Game Boy cartridge reserves a small, fixed region of ROM —
`0x0100`–`0x014F` — for a **header**: structured metadata about the game
itself (title, what hardware features it needs, how big its ROM/RAM are,
etc.). The original hardware's boot ROM reads this header before handing
control to the game. We do the same.

Reference: <https://gbdev.io/pandocs/The_Cartridge_Header.html>

## The header struct

```rust
// src/header.rs
pub struct CartdrigeHeader {
    entry_point: u8,
    nintendo_logo: [u8; 48],
    title: String,
    manufacturer_code: String,
    cgb_flag: String,
    licensee: String,
    sgb_flag: String,
    cartridge_type: String,
    rom_size: String,
    ram_size: String,
    destination_code: String,
    version_number: u8,
    header_checksum: u8,
    global_checksum: u8,
}
```

Each field corresponds to a specific byte range inside the header. Let's
go through the interesting ones.

## The Nintendo logo, and why we validate it

```rust
pub fn is_valid(&self) -> Result<(), String> {
    if self.logo_hex().eq("CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D \
       00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99 BB BB 67 63 6E 0E EC \
       CC DD DC 99 9F BB B9 33 3E") {
        println!("Nintendo Logo is valid");
        Ok(())
    } else {
        Err("Nintendo Logo is invalid".to_string())
    }
}
```

Why bother checking this? On real hardware, the boot ROM actually
**displays** this logo bitmap on screen while booting, and refuses to
continue if the bytes don't match this exact sequence. This was
Nintendo's way of enforcing licensing: cloning a cartridge meant also
illegally copying Nintendo's copyrighted logo bitmap into it. We're not
emulating the boot ROM animation (yet), but we reuse the same check as a
quick sanity test that we're reading a real, intact ROM file.

## Title and manufacturer code: just bytes that happen to be text

```rust
fn parse_title(rom: &[u8]) -> String {
    let title_start = 0x0134;
    let title_end = 0x0143;
    let title_bytes = &rom[title_start..=title_end];
    String::from_utf8_lossy(title_bytes).to_string()
}

fn parse_manufacturercode(rom: &[u8]) -> String {
    let start = 0x013F;
    let end = 0x0142;
    let code = &rom[start..=end];
    String::from_utf8_lossy(code).to_string()
}
```

This is a good moment to internalize something important: **a "byte" has
no inherent meaning** — it's just a number from 0 to 255. Whether a given
byte means "part of an instruction," "part of a picture," or "a text
character" depends entirely on *where* it is and how the code reading it
chooses to interpret it. Here, bytes `0x0134`–`0x0143` are defined (by
convention, by Nintendo) to mean "ASCII text, the game's title" — so we
read them and convert to a `String`. A few bytes earlier and the exact
same kind of raw byte would mean something totally different (part of the
logo bitmap). Keep this in mind for every future chapter.

## Licensee code: old format vs. new format

```rust
fn parse_licensee(rom: &[u8]) -> String {
    let old_code = &rom[0x014B];
    if *old_code == 0x33 {
        let new_code = &rom[0x0144..=0x0145];
        let code_cow = String::from_utf8_lossy(new_code);
        return format!(
            "[NEW] {}",
            NEW_LICENSEE_MAP.get(code_cow.as_ref()).unwrap_or(&"Unknown")
        );
    }
    format!("[OLD] {}", OLD_LICENSEE_MAP.get(old_code).unwrap_or(&"Unknown"))
}
```

This one teaches a recurring Game Boy theme: **backwards compatibility
quirks baked directly into the data format.** Older cartridges store the
publisher ("licensee") as a single byte at `0x014B`, looked up in an old
table. Later, Nintendo ran out of codes, so newer cartridges set
`0x014B = 0x33` as a sentinel meaning "ignore me, the real code is two
ASCII characters over at `0x0144`-`0x0145`, look it up in the *new*
table instead." Both lookup tables (`OLD_LICENSEE_MAP`,
`NEW_LICENSEE_MAP`) are large hardcoded maps in
`src/mappings/licensee_map.rs` — not something you derive, just data you
transcribe from the spec.

## Cartridge type, ROM size, RAM size: more table lookups

```rust
fn parse_cartidge(rom: &[u8]) -> String {
    CARTRIDGE_TYPE_MAP.get(&rom[0x0147]).unwrap_or(&"None").to_string()
}

fn parse_rom_size(rom: &[u8]) -> String {
    ROM_SIZE_MAP.get(&rom[0x0148]).unwrap_or(&"Unknown").to_string()
}

fn parse_ram_size(rom: &[u8]) -> String {
    RAM_SIZE_MAP.get(&rom[0x149]).unwrap_or(&"Unknown").to_string()
}
```

The **cartridge type** byte (`0x0147`) is one of the most important fields
in the whole header: it tells us whether this cartridge is "plain ROM"
(simple, everything fits in the 32KB directly addressable range) or uses a
**Memory Bank Controller** (a little extra chip on the cartridge that lets
games larger than 32KB swap chunks of ROM in and out of the CPU's address
space on demand). We don't act on this information yet — Chapter 8 is
where ROM banking actually gets implemented — but we already parse and
display it here.

## What we have now

- A `CartdrigeHeader` that reads and labels every field of the header
  region, using lookup tables for the fields that are "codes" rather than
  raw numbers or text (cartridge type, licensee, ROM/RAM size).
- Logo validation, as an early "is this a real, undamaged ROM file" check.
- A `print()` method to dump the whole parsed header for debugging —
  invaluable in these early stages, since there's no CPU yet to actually
  run the game and show you if parsing was right.

## What's still missing

- The `entry_point`, `version_number`, `header_checksum`, and
  `global_checksum` fields exist in the struct but aren't actually parsed
  yet at this point in history (they stay at their default `0`) — only
  filled in properly later.
- Nothing *acts* on the cartridge type yet (no ROM banking implementation
  — see Chapter 8).
- Still no CPU. That's next, in Part II.
