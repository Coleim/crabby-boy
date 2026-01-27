pub struct Memory {
    data: [u8; 0x10000],
}

impl Memory {}

// memory.data[0x0000..(0x0000 + rom.len())].copy_from_slice(rom);
// Memory
