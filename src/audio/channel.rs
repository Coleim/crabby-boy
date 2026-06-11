#[derive(Default)]
pub struct Channel {
    pub duty_cycle: u8,
    pub duty_pos: u8,
    // lenght
    pub length_timer: u8,
    pub initial_length_timer: u8,
    pub length_enabled: bool,
    // enveloppe
    pub env_timer: u8,
    pub env_dir: u8,  // 0=down, 1=up
    pub env_pace: u8, // 0..7, vitesse de l'enveloppe

    pub period: u16,
    pub volume: u8, // volume courant (change pendant le jeu)
    pub initial_volume: u8,
    pub freq_timer: u32,
    pub enabled: bool,

    // Sweep
    pub sweep_pace: u8,
    pub sweep_substraction: bool,
    pub sweep_step: u8,
    pub sweep_timer: u8,
    pub sweep_enabled: bool,
}

impl Channel {
    pub fn reset(&mut self) {
        let init_len = self.initial_length_timer;
        let len = self.length_timer;
        *self = Self::default();
        self.initial_length_timer = init_len;
        self.length_timer = len;
    }

    pub fn sweep_next_period_and_overflow(&self) -> (u16, bool) {
        let delta = self.period >> self.sweep_step;
        if self.sweep_substraction {
            (self.period.saturating_sub(delta), false)
        } else {
            let next = self.period as u32 + delta as u32;
            (next as u16, next > 0x7FF)
        }
    }

    pub fn write_sweep(&mut self, val: u8) {
        self.sweep_pace = (val & 0b0111_0000) >> 4;
        self.sweep_substraction = val & 0b0000_1000 != 0;
        self.sweep_step = val & 0b0000_0111;
        self.sweep_timer = self.sweep_pace;

        // Trigger latches whether sweep unit is enabled.
        self.sweep_enabled = self.sweep_pace > 0 || self.sweep_step > 0;
        self.sweep_timer = if self.sweep_pace == 0 {
            8
        } else {
            self.sweep_pace
        };

        // CH1 quirk: if shift>0, trigger performs an immediate sweep calculation.
        if self.sweep_step > 0 {
            let (next, overflow) = self.sweep_next_period_and_overflow();
            if overflow {
                self.enabled = false;
                self.sweep_enabled = false;
            } else {
                self.period = next;
            }
        }
    }

    pub fn write_nr1(&mut self, val: u8) {
        self.duty_cycle = (val & 0b1100_0000) >> 6;
        self.initial_length_timer = 64 - (val & 0b0011_1111);
        self.length_timer = self.initial_length_timer;
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
        self.period = (self.period & 0b111_0000_0000) | val as u16;
    }
    pub fn write_nr4(&mut self, val: u8, length_clock_on_write: bool) {
        let was_length_enabled = self.length_enabled;
        let trigger = val & 0b1000_0000 != 0;

        self.period = (self.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);
        self.length_enabled = val & 0b0100_0000 != 0;

        // DMG quirk: enabling length can immediately clock it depending on frame phase.
        if !was_length_enabled
            && self.length_enabled
            && length_clock_on_write
            && self.length_timer > 0
        {
            self.length_timer = self.length_timer.saturating_sub(1);
            if self.length_timer == 0 && !trigger {
                self.enabled = false;
            }
        }

        if trigger {
            let dac_off = self.initial_volume == 0 && self.env_dir == 0; // (initial volume = 0, envelope = decreasing) turns the DAC off 
            self.enabled = !dac_off;
            if self.length_timer == 0 {
                self.length_timer = 64;
                if self.length_enabled && length_clock_on_write {
                    self.length_timer = self.length_timer.saturating_sub(1);
                }
            }
            self.freq_timer = (2048u32.saturating_sub(self.period as u32) * 4).max(1);
            self.volume = self.initial_volume; // reset volume
            self.env_timer = if self.env_pace == 0 { 8 } else { self.env_pace }; // The volume envelope and sweep timers treat a period of 0 as 8.
        }
    }

    pub fn tick(&mut self) {
        self.freq_timer = self.freq_timer.saturating_sub(1);
        if self.freq_timer == 0 {
            self.freq_timer = (2048u32.saturating_sub(self.period as u32) * 4).max(1);
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }
}
