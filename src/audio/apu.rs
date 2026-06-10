use std::sync::{Arc, Mutex};

use crate::audio::buffer::AudioBuffer;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

const NOISE_DIVISORS: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

const MASTER_VOLUME: f32 = 0.25;
const LP_ALPHA: f32 = 0.4;

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Square {
    sweep_pace: u8,
    sweep_dir: bool,
    sweep_step: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,

    duty: u8,
    duty_pos: u8,
    length_timer: u16,
    length_enabled: bool,

    env_initial: u8,
    env_dir: bool,
    env_pace: u8,
    env_timer: u8,
    volume: u8,

    period: u16,
    freq_timer: i32,
    enabled: bool,
    dac_enabled: bool,
}

impl Square {
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.freq_timer = (2048 - self.period as i32) * 4;
        self.volume = self.env_initial;
        self.env_timer = self.env_pace;
        self.sweep_shadow = self.period;
        self.sweep_timer = if self.sweep_pace != 0 {
            self.sweep_pace
        } else {
            8
        };
        self.sweep_enabled = self.sweep_pace != 0 || self.sweep_step != 0;
        if self.sweep_step != 0 {
            self.sweep_calc();
        }
    }

    fn sweep_calc(&mut self) -> u16 {
        let delta = self.sweep_shadow >> self.sweep_step;
        let new = if self.sweep_dir {
            self.sweep_shadow.wrapping_sub(delta)
        } else {
            self.sweep_shadow + delta
        };
        if new > 2047 {
            self.enabled = false;
        }
        new
    }

    fn clock_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_pace != 0 {
                self.sweep_pace
            } else {
                8
            };
            if self.sweep_enabled && self.sweep_pace != 0 {
                let new = self.sweep_calc();
                if new <= 2047 && self.sweep_step != 0 {
                    self.sweep_shadow = new;
                    self.period = new;
                    self.sweep_calc();
                }
            }
        }
    }

    fn clock_length(&mut self) {
        if self.length_enabled && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    fn clock_envelope(&mut self) {
        if self.env_pace == 0 {
            return;
        }
        if self.env_timer > 0 {
            self.env_timer -= 1;
        }
        if self.env_timer == 0 {
            self.env_timer = self.env_pace;
            if self.env_dir && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_dir && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    fn step(&mut self) {
        self.freq_timer -= 4;
        while self.freq_timer <= 0 {
            self.freq_timer += (2048 - self.period as i32) * 4;
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        DUTY_TABLE[self.duty as usize][self.duty_pos as usize] * self.volume
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Wave {
    dac_enabled: bool,
    length_timer: u16,
    length_enabled: bool,
    volume_code: u8,
    period: u16,
    freq_timer: i32,
    position: u8,
    sample: u8,
    ram: [u8; 16],
    enabled: bool,
}

impl Wave {
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_timer == 0 {
            self.length_timer = 256;
        }
        self.freq_timer = (2048 - self.period as i32) * 2;
        self.position = 0;
    }

    fn clock_length(&mut self) {
        if self.length_enabled && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    fn step(&mut self) {
        self.freq_timer -= 4;
        while self.freq_timer <= 0 {
            self.freq_timer += (2048 - self.period as i32) * 2;
            self.position = (self.position + 1) % 32;
            let byte = self.ram[(self.position / 2) as usize];
            self.sample = if self.position % 2 == 0 {
                byte >> 4
            } else {
                byte & 0x0F
            };
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        match self.volume_code {
            0 => 0,
            1 => self.sample,
            2 => self.sample >> 1,
            _ => self.sample >> 2,
        }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Noise {
    length_timer: u16,
    length_enabled: bool,

    env_initial: u8,
    env_dir: bool,
    env_pace: u8,
    env_timer: u8,
    volume: u8,

    clock_shift: u8,
    width_mode: bool,
    divisor_code: u8,
    freq_timer: i32,
    lfsr: u16,
    enabled: bool,
    dac_enabled: bool,
}

impl Noise {
    fn period(&self) -> i32 {
        (NOISE_DIVISORS[self.divisor_code as usize] << self.clock_shift) as i32
    }

    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_timer == 0 {
            self.length_timer = 64;
        }
        self.freq_timer = self.period();
        self.volume = self.env_initial;
        self.env_timer = self.env_pace;
        self.lfsr = 0x7FFF;
    }

    fn clock_length(&mut self) {
        if self.length_enabled && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    fn clock_envelope(&mut self) {
        if self.env_pace == 0 {
            return;
        }
        if self.env_timer > 0 {
            self.env_timer -= 1;
        }
        if self.env_timer == 0 {
            self.env_timer = self.env_pace;
            if self.env_dir && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_dir && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    fn step(&mut self) {
        self.freq_timer -= 4;
        while self.freq_timer <= 0 {
            self.freq_timer += self.period().max(1);
            let bit = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
            self.lfsr = (self.lfsr >> 1) | (bit << 14);
            if self.width_mode {
                self.lfsr = (self.lfsr & !(1 << 6)) | (bit << 6);
            }
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        ((!self.lfsr & 1) as u8) * self.volume
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct APU {
    #[serde(skip)]
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    frame_counter: u32,
    frame_step: u8,
    sample_counter: f64,
    ticks_per_sample: f64,
    is_on: bool,

    nr50: u8,
    nr51: u8,

    ch1: Square,
    ch2: Square,
    ch3: Wave,
    ch4: Noise,

    hp_prev_in: f32,
    hp_prev_out: f32,
    lp_prev: f32,
}

impl APU {
    const CPU_FREQ: u32 = 4_194_304;

    pub fn new() -> Self {
        APU {
            audio_buffer: None,
            frame_counter: 0,
            frame_step: 0,
            sample_counter: 0.0,
            ticks_per_sample: APU::CPU_FREQ as f64 / 44_100.0,
            is_on: false,
            nr50: 0,
            nr51: 0,
            ch1: Square::default(),
            ch2: Square::default(),
            ch3: Wave::default(),
            ch4: Noise::default(),
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            lp_prev: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.ticks_per_sample = APU::CPU_FREQ as f64 / sample_rate.max(1) as f64;
    }

    pub fn set_audio_buffer(&mut self, buffer: Arc<Mutex<AudioBuffer>>) {
        self.audio_buffer = Some(buffer);
    }

    pub fn audio_buffer_handle(&self) -> Option<Arc<Mutex<AudioBuffer>>> {
        self.audio_buffer.clone()
    }

    pub fn tick(&mut self) {
        if self.is_on {
            self.frame_counter += 4;
            if self.frame_counter >= 8192 {
                self.frame_counter -= 8192;
                self.clock_frame_sequencer();
            }
            self.ch1.step();
            self.ch2.step();
            self.ch3.step();
            self.ch4.step();
        }

        self.sample_counter += 4.0;
        if self.sample_counter >= self.ticks_per_sample {
            self.sample_counter -= self.ticks_per_sample;
            let sample = self.mix();
            if let Some(buffer) = &self.audio_buffer {
                buffer.lock().unwrap().push(sample);
            }
        }
    }

    fn clock_frame_sequencer(&mut self) {
        match self.frame_step {
            0 | 4 => self.clock_length(),
            2 | 6 => {
                self.clock_length();
                self.ch1.clock_sweep();
            }
            7 => {
                self.ch1.clock_envelope();
                self.ch2.clock_envelope();
                self.ch4.clock_envelope();
            }
            _ => {}
        }
        self.frame_step = (self.frame_step + 1) % 8;
    }

    fn clock_length(&mut self) {
        self.ch1.clock_length();
        self.ch2.clock_length();
        self.ch3.clock_length();
        self.ch4.clock_length();
    }

    fn dac(level: u8) -> f32 {
        level as f32 / 7.5 - 1.0
    }

    fn mix(&mut self) -> f32 {
        if !self.is_on {
            return 0.0;
        }

        let outs = [
            (self.ch1.output(), self.ch1.dac_enabled),
            (self.ch2.output(), self.ch2.dac_enabled),
            (self.ch3.output(), self.ch3.dac_enabled),
            (self.ch4.output(), self.ch4.dac_enabled),
        ];

        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for (i, (level, dac)) in outs.iter().enumerate() {
            if !dac {
                continue;
            }
            let analog = Self::dac(*level);
            if self.nr51 & (1 << (i + 4)) != 0 {
                left += analog;
            }
            if self.nr51 & (1 << i) != 0 {
                right += analog;
            }
        }

        let left_vol = ((self.nr50 >> 4) & 0x7) as f32 + 1.0;
        let right_vol = (self.nr50 & 0x7) as f32 + 1.0;
        left = left * left_vol / 8.0 / 4.0;
        right = right * right_vol / 8.0 / 4.0;

        let mono = (left + right) * 0.5 * MASTER_VOLUME;
        let hp = mono - self.hp_prev_in + 0.996 * self.hp_prev_out;
        self.hp_prev_in = mono;
        self.hp_prev_out = hp;
        self.lp_prev += LP_ALPHA * (hp - self.lp_prev);
        self.lp_prev.clamp(-1.0, 1.0)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.read_nr10(),
            0xFF11 => (self.ch1.duty << 6) | 0x3F,
            0xFF12 => self.read_env(self.ch1.env_initial, self.ch1.env_dir, self.ch1.env_pace),
            0xFF13 => 0xFF,
            0xFF14 => (self.ch1.length_enabled as u8) << 6 | 0xBF,
            0xFF16 => (self.ch2.duty << 6) | 0x3F,
            0xFF17 => self.read_env(self.ch2.env_initial, self.ch2.env_dir, self.ch2.env_pace),
            0xFF18 => 0xFF,
            0xFF19 => (self.ch2.length_enabled as u8) << 6 | 0xBF,
            0xFF1A => (self.ch3.dac_enabled as u8) << 7 | 0x7F,
            0xFF1B => 0xFF,
            0xFF1C => (self.ch3.volume_code << 5) | 0x9F,
            0xFF1D => 0xFF,
            0xFF1E => (self.ch3.length_enabled as u8) << 6 | 0xBF,
            0xFF20 => 0xFF,
            0xFF21 => self.read_env(self.ch4.env_initial, self.ch4.env_dir, self.ch4.env_pace),
            0xFF22 => {
                (self.ch4.clock_shift << 4)
                    | (self.ch4.width_mode as u8) << 3
                    | self.ch4.divisor_code
            }
            0xFF23 => (self.ch4.length_enabled as u8) << 6 | 0xBF,
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => self.read_nr52(),
            0xFF30..=0xFF3F => self.ch3.ram[(addr - 0xFF30) as usize],
            _ => 0xFF,
        }
    }

    fn read_nr10(&self) -> u8 {
        0x80 | (self.ch1.sweep_pace << 4) | (self.ch1.sweep_dir as u8) << 3 | self.ch1.sweep_step
    }

    fn read_env(&self, initial: u8, dir: bool, pace: u8) -> u8 {
        (initial << 4) | (dir as u8) << 3 | pace
    }

    fn read_nr52(&self) -> u8 {
        let mut status = 0x70;
        if self.is_on {
            status |= 0x80;
        }
        if self.ch1.enabled {
            status |= 0x01;
        }
        if self.ch2.enabled {
            status |= 0x02;
        }
        if self.ch3.enabled {
            status |= 0x04;
        }
        if self.ch4.enabled {
            status |= 0x08;
        }
        status
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr == 0xFF26 {
            self.is_on = val & 0x80 != 0;
            if !self.is_on {
                self.power_off();
            }
            return;
        }

        if (0xFF30..=0xFF3F).contains(&addr) {
            self.ch3.ram[(addr - 0xFF30) as usize] = val;
            return;
        }

        if !self.is_on {
            return;
        }

        match addr {
            0xFF10 => {
                self.ch1.sweep_pace = (val >> 4) & 0x7;
                self.ch1.sweep_dir = val & 0x08 != 0;
                self.ch1.sweep_step = val & 0x07;
            }
            0xFF11 => {
                self.ch1.duty = val >> 6;
                self.ch1.length_timer = 64 - (val & 0x3F) as u16;
            }
            0xFF12 => self.write_square_env(true, val),
            0xFF13 => self.ch1.period = (self.ch1.period & 0x700) | val as u16,
            0xFF14 => {
                self.ch1.period = (self.ch1.period & 0xFF) | ((val as u16 & 0x7) << 8);
                self.ch1.length_enabled = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch1.trigger();
                }
            }
            0xFF16 => {
                self.ch2.duty = val >> 6;
                self.ch2.length_timer = 64 - (val & 0x3F) as u16;
            }
            0xFF17 => self.write_square_env(false, val),
            0xFF18 => self.ch2.period = (self.ch2.period & 0x700) | val as u16,
            0xFF19 => {
                self.ch2.period = (self.ch2.period & 0xFF) | ((val as u16 & 0x7) << 8);
                self.ch2.length_enabled = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch2.trigger();
                }
            }
            0xFF1A => {
                self.ch3.dac_enabled = val & 0x80 != 0;
                if !self.ch3.dac_enabled {
                    self.ch3.enabled = false;
                }
            }
            0xFF1B => self.ch3.length_timer = 256 - val as u16,
            0xFF1C => self.ch3.volume_code = (val >> 5) & 0x3,
            0xFF1D => self.ch3.period = (self.ch3.period & 0x700) | val as u16,
            0xFF1E => {
                self.ch3.period = (self.ch3.period & 0xFF) | ((val as u16 & 0x7) << 8);
                self.ch3.length_enabled = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch3.trigger();
                }
            }
            0xFF20 => self.ch4.length_timer = 64 - (val & 0x3F) as u16,
            0xFF21 => {
                self.ch4.env_initial = val >> 4;
                self.ch4.env_dir = val & 0x08 != 0;
                self.ch4.env_pace = val & 0x07;
                self.ch4.dac_enabled = val & 0xF8 != 0;
                if !self.ch4.dac_enabled {
                    self.ch4.enabled = false;
                }
            }
            0xFF22 => {
                self.ch4.clock_shift = val >> 4;
                self.ch4.width_mode = val & 0x08 != 0;
                self.ch4.divisor_code = val & 0x07;
            }
            0xFF23 => {
                self.ch4.length_enabled = val & 0x40 != 0;
                if val & 0x80 != 0 {
                    self.ch4.trigger();
                }
            }
            0xFF24 => self.nr50 = val,
            0xFF25 => self.nr51 = val,
            _ => {}
        }
    }

    fn write_square_env(&mut self, ch1: bool, val: u8) {
        let ch = if ch1 { &mut self.ch1 } else { &mut self.ch2 };
        ch.env_initial = val >> 4;
        ch.env_dir = val & 0x08 != 0;
        ch.env_pace = val & 0x07;
        ch.dac_enabled = val & 0xF8 != 0;
        if !ch.dac_enabled {
            ch.enabled = false;
        }
    }

    fn power_off(&mut self) {
        let wave_ram = self.ch3.ram;
        self.ch1 = Square::default();
        self.ch2 = Square::default();
        self.ch3 = Wave::default();
        self.ch3.ram = wave_ram;
        self.ch4 = Noise::default();
        self.nr50 = 0;
        self.nr51 = 0;
        self.frame_step = 0;
    }
}
