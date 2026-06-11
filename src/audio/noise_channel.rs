#[derive(Default)]
pub struct NoiseChannel {
    pub enabled: bool,
    pub freq_timer: u32,
    pub initial_len_timer: u16,
    pub len_timer: u16,
    pub length_enabled: bool,
    // enveloppe
    pub env_timer: u8,
    pub env_dir: u8,  // 0=down, 1=up
    pub env_pace: u8, // 0..7, vitesse de l'enveloppe

    pub volume: u8, // volume courant (change pendant le jeu)
    pub initial_volume: u8,

    pub clock_shift: u8,
    pub clock_divider: u8,
    pub short_mode: bool,
    pub lfsr: u16,
}
impl NoiseChannel {
    pub fn reset(&mut self) {
        let init_len = self.initial_len_timer;
        let len = self.len_timer;
        *self = Self::default();
        self.initial_len_timer = init_len;
        self.len_timer = len;
    }

    pub fn write_nr1(&mut self, val: u8) {
        self.initial_len_timer = 64 - (val & 0b0011_1111) as u16;
        self.len_timer = self.initial_len_timer;
    }

    pub fn write_nr2(&mut self, val: u8) {
        self.initial_volume = (val & 0b1111_0000) >> 4; // bits 7-4
        self.env_dir = (val & 0b0000_1000) >> 3; // bit 3
        self.env_pace = val & 0b0000_0111; // bits 2-0
        let dac_off = self.initial_volume == 0 && self.env_dir == 0; // (initial volume = 0, envelope = decreasing) turns the DAC off 
        if dac_off {
            self.enabled = false;
        }
    }

    pub fn write_nr3(&mut self, val: u8) {
        self.clock_shift = (val & 0b1111_0000) >> 4;
        self.clock_divider = val & 0b0000_0111;
        self.short_mode = (val & 0b0000_1000) != 0;
    }

    pub fn write_nr4(&mut self, val: u8, length_clock_on_write: bool) {
        let was_length_enabled = self.length_enabled;
        let trigger = val & 0b1000_0000 != 0;

        // DMG quirk: enabling length can immediately clock it depending on frame phase.
        if !was_length_enabled && self.length_enabled && length_clock_on_write && self.len_timer > 0
        {
            self.len_timer = self.len_timer.saturating_sub(1);
            if self.len_timer == 0 && !trigger {
                self.enabled = false;
            }
        }

        self.length_enabled = val & 0b0100_0000 != 0;
        if trigger {
            let dac_off = self.initial_volume == 0 && self.env_dir == 0; // (initial volume = 0, envelope = decreasing) turns the DAC off 
            self.enabled = !dac_off;
            if self.len_timer == 0 {
                self.len_timer = 64;
                if self.length_enabled && length_clock_on_write {
                    self.len_timer = self.len_timer.saturating_sub(1);
                }
            }
            self.volume = self.initial_volume; // reset volume
            self.env_timer = if self.env_pace == 0 { 8 } else { self.env_pace }; // The volume envelope and sweep timers treat a period of 0 as 8.
            // LFSR bits are reset.
            self.lfsr = 0;
            let divider = if self.clock_divider == 0 {
                8
            } else {
                self.clock_divider as u32 * 16
            };
            self.freq_timer = divider << self.clock_shift;
        }
    }

    pub fn tick(&mut self) {
        self.freq_timer = self.freq_timer.saturating_sub(1);
        if self.freq_timer == 0 {
            let divider = if self.clock_divider == 0 {
                8
            } else {
                self.clock_divider as u32 * 16
            };
            self.freq_timer = divider << self.clock_shift;

            // The result of LFSR0 ⊙ LFSR1 (1 if bit 0 and bit 1 are identical, 0 otherwise) is written to bit 15.

            let bit0 = self.lfsr & 0x01;
            let bit1 = (self.lfsr >> 1) & 0x01;
            let res = if bit0 == bit1 { 1 } else { 0 };
            self.lfsr = (self.lfsr & 0b0111_1111_1111_1111) | (res << 15);

            // If “short mode” was selected in NR43, then bit 15 is copied to bit 7 as well.
            if self.short_mode {
                self.lfsr = (self.lfsr & 0b1111_1111_0111_1111) | (res << 7);
            }

            // Finally, the entire LFSR is shifted right, and bit 0 selects between 0 and the chosen volume.
            self.lfsr = self.lfsr >> 1;
        }
    }
}
