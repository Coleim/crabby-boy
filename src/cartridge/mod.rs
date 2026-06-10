pub mod header;
pub mod mappings;

#[derive(serde::Serialize, serde::Deserialize)]
enum Mbc {
    None,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cartridge {
    #[serde(skip)]
    rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: Mbc,
    has_ram: bool,
    has_battery: bool,
    rom_banks: usize,
    ram_banks: usize,

    ram_enabled: bool,
    rom_bank: usize,
    ram_bank: usize,
    banking_mode: u8,

    rtc: [u8; 5],
    rtc_latch: [u8; 5],
    rtc_select: u8,
    rtc_latch_armed: bool,
    has_rtc: bool,
    rtc_cycles: u32,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let cart_type = rom.get(0x0147).copied().unwrap_or(0);
        let (mbc, has_ram, has_battery) = Self::decode_type(cart_type);
        let has_rtc = matches!(cart_type, 0x0F | 0x10);

        let rom_banks = (rom.len() / 0x4000).max(2);
        let ram_size = match rom.get(0x0149).copied().unwrap_or(0) {
            0x02 => 0x2000,
            0x03 => 0x8000,
            0x04 => 0x20000,
            0x05 => 0x10000,
            _ => 0,
        };
        let ram_len = match mbc {
            Mbc::Mbc2 => 0x200,
            _ if has_ram => ram_size.max(0x2000),
            _ => ram_size,
        };
        let ram_banks = (ram_len / 0x2000).max(1);

        Cartridge {
            rom,
            ram: vec![0; ram_len],
            mbc,
            has_ram,
            has_battery,
            rom_banks,
            ram_banks,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: 0,
            rtc: [0; 5],
            rtc_latch: [0; 5],
            rtc_select: 0,
            rtc_latch_armed: false,
            has_rtc,
            rtc_cycles: 0,
        }
    }

    pub fn tick(&mut self) {
        if !self.has_rtc || self.rtc[4] & 0x40 != 0 {
            return;
        }
        self.rtc_cycles += 1;
        if self.rtc_cycles < 1_048_576 {
            return;
        }
        self.rtc_cycles = 0;
        self.advance_rtc();
    }

    fn advance_rtc(&mut self) {
        self.rtc[0] += 1;
        if self.rtc[0] < 60 {
            return;
        }
        self.rtc[0] = 0;
        self.rtc[1] += 1;
        if self.rtc[1] < 60 {
            return;
        }
        self.rtc[1] = 0;
        self.rtc[2] += 1;
        if self.rtc[2] < 24 {
            return;
        }
        self.rtc[2] = 0;
        let day = (((self.rtc[4] as u16 & 1) << 8) | self.rtc[3] as u16) + 1;
        self.rtc[3] = day as u8;
        self.rtc[4] = (self.rtc[4] & !1) | ((day >> 8) & 1) as u8;
        if day > 511 {
            self.rtc[4] = (self.rtc[4] & !1) | 0x80;
            self.rtc[3] = 0;
        }
    }

    fn decode_type(t: u8) -> (Mbc, bool, bool) {
        match t {
            0x00 => (Mbc::None, false, false),
            0x01 => (Mbc::Mbc1, false, false),
            0x02 => (Mbc::Mbc1, true, false),
            0x03 => (Mbc::Mbc1, true, true),
            0x05 => (Mbc::Mbc2, true, false),
            0x06 => (Mbc::Mbc2, true, true),
            0x08 => (Mbc::None, true, false),
            0x09 => (Mbc::None, true, true),
            0x0F => (Mbc::Mbc3, false, true),
            0x10 => (Mbc::Mbc3, true, true),
            0x11 => (Mbc::Mbc3, false, false),
            0x12 => (Mbc::Mbc3, true, false),
            0x13 => (Mbc::Mbc3, true, true),
            0x19 | 0x1A | 0x1C | 0x1D => (Mbc::Mbc5, true, false),
            0x1B | 0x1E => (Mbc::Mbc5, true, true),
            _ => (Mbc::Mbc1, true, false),
        }
    }

    fn rom_mask(&self) -> usize {
        self.rom_banks - 1
    }

    pub fn read_rom(&self, addr: u16) -> u8 {
        let bank = match addr {
            0x0000..=0x3FFF => match self.mbc {
                Mbc::Mbc1 if self.banking_mode == 1 => (self.ram_bank << 5) & self.rom_mask(),
                _ => 0,
            },
            _ => self.rom_bank & self.rom_mask(),
        };
        let offset = bank * 0x4000 + (addr as usize & 0x3FFF);
        self.rom.get(offset).copied().unwrap_or(0xFF)
    }

    pub fn write_control(&mut self, addr: u16, val: u8) {
        match self.mbc {
            Mbc::None => {}
            Mbc::Mbc1 => self.write_mbc1(addr, val),
            Mbc::Mbc2 => self.write_mbc2(addr, val),
            Mbc::Mbc3 => self.write_mbc3(addr, val),
            Mbc::Mbc5 => self.write_mbc5(addr, val),
        }
    }

    fn write_mbc1(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                let low = (val & 0x1F) as usize;
                self.rom_bank = (self.rom_bank & 0x60) | if low == 0 { 1 } else { low };
            }
            0x4000..=0x5FFF => {
                let high = (val & 0x03) as usize;
                self.ram_bank = high;
                self.rom_bank = (self.rom_bank & 0x1F) | (high << 5);
            }
            0x6000..=0x7FFF => self.banking_mode = val & 0x01,
            _ => {}
        }
    }

    fn write_mbc2(&mut self, addr: u16, val: u8) {
        if addr <= 0x3FFF {
            if addr & 0x0100 == 0 {
                self.ram_enabled = val & 0x0F == 0x0A;
            } else {
                let low = (val & 0x0F) as usize;
                self.rom_bank = if low == 0 { 1 } else { low };
            }
        }
    }

    fn write_mbc3(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                let bank = (val & 0x7F) as usize;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => self.rtc_select = val,
            0x6000..=0x7FFF => {
                if self.rtc_latch_armed && val == 1 {
                    self.rtc_latch = self.rtc;
                }
                self.rtc_latch_armed = val == 0;
            }
            _ => {}
        }
    }

    fn write_mbc5(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = val & 0x0F == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | val as usize,
            0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0xFF) | ((val as usize & 1) << 8),
            0x4000..=0x5FFF => self.ram_bank = (val & 0x0F) as usize,
            _ => {}
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        if let Mbc::Mbc3 = self.mbc {
            if self.rtc_select >= 0x08 && self.rtc_select <= 0x0C {
                return self.rtc_latch[(self.rtc_select - 0x08) as usize];
            }
        }
        if let Mbc::Mbc2 = self.mbc {
            return self.ram[addr as usize & 0x1FF] | 0xF0;
        }
        if self.ram.is_empty() {
            return 0xFF;
        }
        let offset = self.ram_offset(addr);
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        if !self.ram_enabled {
            return;
        }
        if let Mbc::Mbc3 = self.mbc {
            if self.rtc_select >= 0x08 && self.rtc_select <= 0x0C {
                self.rtc[(self.rtc_select - 0x08) as usize] = val;
                return;
            }
        }
        if let Mbc::Mbc2 = self.mbc {
            self.ram[addr as usize & 0x1FF] = val & 0x0F;
            return;
        }
        if self.ram.is_empty() {
            return;
        }
        let offset = self.ram_offset(addr);
        if offset < self.ram.len() {
            self.ram[offset] = val;
        }
    }

    fn ram_offset(&self, addr: u16) -> usize {
        let bank = match self.mbc {
            Mbc::Mbc1 if self.banking_mode == 1 => self.ram_bank,
            Mbc::Mbc1 => 0,
            _ => self.ram_bank,
        };
        let bank = bank % self.ram_banks.max(1);
        bank * 0x2000 + (addr as usize - 0xA000)
    }

    #[cfg(test)]
    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn take_rom(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.rom)
    }

    pub fn set_rom(&mut self, rom: Vec<u8>) {
        self.rom = rom;
    }

    pub fn battery_ram(&self) -> Option<&[u8]> {
        if self.has_battery && self.has_ram && !self.ram.is_empty() {
            Some(&self.ram)
        } else {
            None
        }
    }

    pub fn load_ram(&mut self, data: &[u8]) {
        if self.has_battery && self.has_ram && data.len() == self.ram.len() {
            self.ram.copy_from_slice(data);
        }
    }
}
