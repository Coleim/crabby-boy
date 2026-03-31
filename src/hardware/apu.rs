pub struct APU {
    nr52: u8, // FF26 — NR52: Audio master control
    nr51: u8, // FF25 — NR51: Sound panning
    nr50: u8, // FF24 — NR50: Master volume & VIN panning
              // FF10 — NR10: Channel 1 sweep
              // nr44: u8, // FF23 — NR44: Channel 4 control
}

impl APU {
    pub fn new() -> Self {
        APU {
            nr52: 0,
            nr51: 0,
            nr50: 0,
            // nr44: 0,
        }
    }

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
            // 0xFF23 => self.nr44 = val,
            _ => {
                println!("[AUDIO REG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }
}
