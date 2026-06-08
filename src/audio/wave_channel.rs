#[derive(Default)]
pub struct WaveChannel {
    pub dac_enabled: bool,
    pub enabled: bool,
    pub initial_len_timer: u16,
    pub len_timer: u16,
    pub length_enabled: bool,
    pub freq_timer: u32,
    pub volume_level: u8,
    pub period: u16,
    pub wave_ram: [u8; 16],
    pub wave_index: u8, // 0..31 (32 nibbles total)
}

impl WaveChannel {
    pub fn write_nr0(&mut self, val: u8) {
        self.dac_enabled = val & 0b1000_0000 != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }
    pub fn write_nr1(&mut self, val: u8) {
        self.initial_len_timer = 256 - val as u16;
        self.len_timer = self.initial_len_timer;
    }
    pub fn write_nr2(&mut self, val: u8) {
        self.volume_level = (val & 0b0110_0000) >> 5;
    }

    pub fn write_nr3(&mut self, val: u8) {
        self.period = (self.period & 0b111_0000_0000) | val as u16;
    }

    pub fn write_nr4(&mut self, val: u8) {
        self.period = (self.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);
        self.length_enabled = val & 0b0100_0000 != 0;
        if val & 0b1000_0000 != 0 {
            self.enabled = true;
            if self.len_timer == 0 {
                self.len_timer = 256;
            }
            self.freq_timer = (2048 - self.period as u32) * 2; // CH3 uses *2 not *4
            self.wave_index = 1;
        }
    }

    pub fn read_wave_ram(&self, addr: u8) -> u8 {
        if self.enabled {
            return 0xFF; // locked while playing (DMG behavior)
        }
        self.wave_ram[addr as usize]
    }

    pub fn write_wave_ram(&mut self, addr: u8, val: u8) {
        if self.enabled {
            return; // ignore writes while playing (DMG behavior)
        }
        self.wave_ram[addr as usize] = val;
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.freq_timer = self.freq_timer.saturating_sub(1);
        if self.freq_timer == 0 {
            self.freq_timer = (2048 - self.period as u32) * 2;
            self.wave_index = (self.wave_index + 1) % 32;
        }
    }
}
