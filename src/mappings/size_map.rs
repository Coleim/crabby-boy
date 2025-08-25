use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static ROM_SIZE_MAP: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0x00, "32 KiB (2 banks)");
    m.insert(0x01, "64 KiB (4 banks)");
    m.insert(0x02, "128 KiB (8 banks)");
    m.insert(0x03, "256 KiB (16 banks)");
    m.insert(0x04, "512 KiB (32 banks)");
    m.insert(0x05, "1 MiB (64 banks)");
    m.insert(0x06, "2 MiB (128 banks)");
    m.insert(0x07, "4 MiB (256 banks)");
    m.insert(0x08, "8 MiB (512 banks)");
    m.insert(0x52, "1.1 MiB (72 banks)");
    m.insert(0x53, "1.2 MiB (80 banks)");
    m.insert(0x54, "1.5 MiB (96 banks)");
    m.insert(0xFF, "HuC1+RAM+BATTERY"); // keep your original special entry
    m
});

pub static RAM_SIZE_MAP: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0x00, "0 (No RAM)");
    m.insert(0x01, "Unused");
    m.insert(0x02, "8 KiB (1 bank)");
    m.insert(0x03, "32 KiB (4 banks of 8 KiB)");
    m.insert(0x04, "128 KiB (16 banks of 8 KiB)");
    m.insert(0x05, "64 KiB (8 banks of 8 KiB)");
    m
});
