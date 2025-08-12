pub struct Memory {
    data: [u8; 0x10000],
}

impl Memory {
    pub fn new(rom: &[u8]) -> Self {
        let mut memory = Memory { data: [0; 0x10000] };
        memory.data[0x0000..(0x0000 + rom.len())].copy_from_slice(rom);
        memory
    }
}

// memory.data[0x0000..(0x0000 + rom.len())].copy_from_slice(rom);
// Memory
