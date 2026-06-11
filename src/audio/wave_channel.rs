#[derive(Default)]
pub struct WaveChannel {
    pub dac_enabled: bool,
    pub enabled: bool,
    pub initial_len_timer: u16,
    pub len_timer: u16,
    pub length_enabled: bool,
    pub sample_countdown: u16,

    pub volume_level: u8,
    pub period: u16,
    pub wave_ram: [u8; 16],
    pub wave_index: u8, // 0..31 (32 nibbles total)
    //
    pub sample_buffer: u8,
    pub divider_phase: bool,
    pub wave_form_just_read: bool,
    pub wave_access_window: u8,
}

impl WaveChannel {
    pub fn reset(&mut self) {
        let init_len = self.initial_len_timer;
        let len = self.len_timer;
        let wave_ram = self.wave_ram;
        *self = Self::default();
        self.initial_len_timer = init_len;
        self.len_timer = len;
        self.wave_ram = wave_ram;
    }

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

    pub fn write_nr4(&mut self, val: u8, length_clock_on_write: bool) {
        let was_length_enabled = self.length_enabled;
        let trigger = val & 0b1000_0000 != 0;

        self.period = (self.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);
        self.length_enabled = val & 0b0100_0000 != 0;

        // DMG quirk: enabling length can immediately clock it depending on frame phase.
        if !was_length_enabled && self.length_enabled && length_clock_on_write && self.len_timer > 0
        {
            self.len_timer = self.len_timer.saturating_sub(1);
            if self.len_timer == 0 && !trigger {
                self.enabled = false;
            }
        }

        if trigger {
            // DMG quirk: retriggering CH3 while active right as a sample byte is fetched
            // corrupts wave RAM's first bytes.
            if self.enabled && self.sample_countdown == 0 {
                let offset = ((self.wave_index.wrapping_add(1) >> 1) & 0x0F) as usize;
                if offset < 4 {
                    self.wave_ram[0] = self.wave_ram[offset];
                } else {
                    let start = offset & !0x03;
                    for i in 0..4 {
                        self.wave_ram[i] = self.wave_ram[start + i];
                    }
                }
            }

            self.enabled = self.dac_enabled;
            if self.len_timer == 0 {
                self.len_timer = 256;
                if self.length_enabled && length_clock_on_write {
                    self.len_timer = self.len_timer.saturating_sub(1);
                }
            }

            self.sample_countdown = (self.period ^ 0x07FF) + 3;
            self.wave_index = 0;
            self.divider_phase = false;
            self.wave_form_just_read = false;
            self.wave_access_window = 0;
        }
    }

    pub fn read_wave_ram(&self, addr: u8) -> u8 {
        if self.enabled {
            if self.wave_access_window == 0 {
                return 0xFF;
            }
            return self.wave_ram[(self.wave_index / 2) as usize];
        }
        self.wave_ram[addr as usize]
    }

    pub fn write_wave_ram(&mut self, addr: u8, val: u8) {
        if self.enabled {
            if self.wave_access_window == 0 {
                return;
            }
            let current_byte = (self.wave_index / 2) as usize;
            self.wave_ram[current_byte] = val;
            return;
        }
        self.wave_ram[addr as usize] = val;
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.wave_form_just_read = false;
        if self.wave_access_window > 0 {
            self.wave_access_window -= 1;
        }

        self.divider_phase = !self.divider_phase;
        if !self.divider_phase {
            return;
        }

        if self.sample_countdown > 0 {
            self.sample_countdown -= 1;
            return;
        }

        self.sample_countdown = self.period ^ 0x07FF;
        self.wave_index = (self.wave_index + 1) & 0x1F;

        let byte = self.wave_ram[(self.wave_index / 2) as usize];
        self.sample_buffer = if self.wave_index & 1 == 0 {
            (byte >> 4) & 0x0F
        } else {
            byte & 0x0F
        };
        self.wave_form_just_read = true;
        self.wave_access_window = 2;
    }
}
