use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static CARTRIDGE_TYPE_MAP: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0x00, "ROM ONLY");
    m.insert(0x01, "MBC1");
    m.insert(0x02, "MBC1+RAM");
    m.insert(0x03, "MBC1+RAM+BATTERY");
    m.insert(0x05, "MBC2");
    m.insert(0x06, "MBC2+BATTERY");
    m.insert(0x08, "ROM+RAM 9");
    m.insert(0x09, "ROM+RAM+BATTERY 9");
    m.insert(0x0B, "MMM01");
    m.insert(0x0C, "MMM01+RAM");
    m.insert(0x0D, "MMM01+RAM+BATTERY");
    m.insert(0x0F, "MBC3+TIMER+BATTERY");
    m.insert(0x10, "MBC3+TIMER+RAM+BATTERY 10");
    m.insert(0x11, "MBC3");
    m.insert(0x12, "MBC3+RAM 10");
    m.insert(0x13, "MBC3+RAM+BATTERY 10");
    m.insert(0x19, "MBC5");
    m.insert(0x1A, "MBC5+RAM");
    m.insert(0x1B, "MBC5+RAM+BATTERY");
    m.insert(0x1C, "MBC5+RUMBLE");
    m.insert(0x1D, "MBC5+RUMBLE+RAM");
    m.insert(0x1E, "MBC5+RUMBLE+RAM+BATTERY");
    m.insert(0x20, "MBC6");
    m.insert(0x22, "MBC7+SENSOR+RUMBLE+RAM+BATTERY");
    m.insert(0xFC, "POCKET CAMERA");
    m.insert(0xFD, "BANDAI TAMA5");
    m.insert(0xFE, "HuC3");
    m.insert(0xFF, "HuC1+RAM+BATTERY");
    m
});
