pub struct APU {
    nr52: u8, // FF26 — NR52: Audio master control
    nr51: u8, // FF25 — NR51: Sound panning
    nr50: u8, // FF24 — NR50: Master volume & VIN panning
    // FF10 — NR10: Channel 1 sweep
    // nr44: u8, // FF23 — NR44: Channel 4 control
    nr11: u8,
    nr12: u8,
    nr14: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
}

impl APU {
    pub fn new() -> Self {
        APU {
            nr52: 0,
            nr51: 0,
            nr50: 0,
            nr11: 0,
            nr12: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            // nr44: 0,
        }
    }

    pub fn tick(&mut self) {}

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF26 => self.nr52,
            0xFF25 => self.nr51,
            0xFF24 => self.nr50,
            // 0xFF23 => self.nr44,
            _ => {
                println!("[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF26 => self.nr52 = val, //TODO: self.nr52 = val & 0x80, // only bit 7 writable
            0xFF25 => self.nr51 = val,
            0xFF24 => self.nr50 = val,
            0xFF11 => self.nr11 = val,
            0xFF12 => self.nr12 = val,
            0xFF14 => self.nr14 = val,
            0xFF16 => self.nr21 = val,
            0xFF17 => self.nr22 = val,
            0xFF18 => self.nr23 = val,
            0xFF19 => self.nr24 = val,
            // 0xFF23 => self.nr44 = val,
            _ => {
                println!("[AUDIO REG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                // std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }
}
