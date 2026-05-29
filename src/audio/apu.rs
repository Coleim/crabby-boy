use std::sync::{Arc, Mutex};

use crate::audio::audio_buffer::AudioBuffer;

pub struct Channel {
    // Channel 1
    duty_cycle: u8,
    duty_pos: u8,
    length_timer: u8,
    period: u16,
    volume: u8, // volume courant (change pendant le jeu)
    initial_volume: u8,
    env_dir: u8,  // 0=down, 1=up
    env_pace: u8, // 0..7, vitesse de l'enveloppe
    freq_timer: u32,
    enabled: bool,
}
pub struct APU {
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    cycle_count: u32,
    tick_counter: f64,
    channel1: Channel,
    nr52: u8, // FF26 — NR52: Audio master control
    nr51: u8, // FF25 — NR51: Sound panning
    nr50: u8, // FF24 — NR50: Master volume & VIN panning
    // FF10 — NR10: Channel 1 sweep
    // nr44: u8, // FF23 — NR44: Channel 4 control
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
}

impl APU {
    const CPU_FREQ: u32 = 4_194_304;
    const SAMPLE_RATE: u32 = 44_100;

    // Tous les combien de T-cycles on capture un sample
    const TICKS_PER_SAMPLE: f64 = APU::CPU_FREQ as f64 / APU::SAMPLE_RATE as f64;

    const DUTY_TABLE: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1], // 00 → 12.5%
        [1, 0, 0, 0, 0, 0, 0, 1], // 01 → 25%
        [1, 0, 0, 0, 0, 1, 1, 1], // 10 → 50%
        [0, 1, 1, 1, 1, 1, 1, 0], // 11 → 75%
    ];

    pub fn new() -> Self {
        APU {
            audio_buffer: None,
            cycle_count: 0,
            tick_counter: 0.0,
            channel1: Channel {
                duty_cycle: 0,
                duty_pos: 0,
                length_timer: 0,
                period: 0,
                volume: 0,
                initial_volume: 0,
                env_dir: 0,
                env_pace: 0,
                freq_timer: 0,
                enabled: false,
            },
            nr52: 0,
            nr51: 0,
            nr50: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
        }
    }

    pub fn tick(&mut self) {
        self.cycle_count += 4;

        self.tick_channel1();
        // self.tick_channel2();
        // self.tick_channel3();
        // self.tick_channel4();

        self.tick_counter += 4.0; // accumulateur downsampling
        if self.tick_counter >= APU::TICKS_PER_SAMPLE {
            self.tick_counter -= APU::TICKS_PER_SAMPLE;
            let sample = self.get_sample();
            if let Some(buffer) = &self.audio_buffer {
                buffer.lock().unwrap().push(sample);
            }

            // self.producer.push(self.mix_channels());
        }
    }

    fn tick_channel1(&mut self) {
        self.channel1.freq_timer = self.channel1.freq_timer.saturating_sub(1);
        if self.channel1.freq_timer == 0 {
            self.channel1.freq_timer = (2048 - self.channel1.period as u32) * 4;
            self.channel1.duty_pos = (self.channel1.duty_pos + 1) % 8;
        }
    }

    fn get_sample(&self) -> f32 {
        if !self.channel1.enabled {
            return 0.0;
        }
        let duty =
            APU::DUTY_TABLE[self.channel1.duty_cycle as usize][self.channel1.duty_pos as usize];
        duty as f32 * self.channel1.volume as f32 / 15.0
    }

    pub fn set_audio_buffer(&mut self, buffer: Arc<Mutex<AudioBuffer>>) {
        self.audio_buffer = Some(buffer);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF26 => self.nr52,
            0xFF25 => self.nr51,
            0xFF24 => self.nr50,
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
            // Channel 1
            0xFF11 => {
                self.channel1.duty_cycle = (val & 0b1100_0000) >> 6;
                self.channel1.length_timer = 64 - (val & 0b0011_1111);
            }
            0xFF12 => {
                self.channel1.initial_volume = (val & 0b1111_0000) >> 4; // bits 7-4
                self.channel1.env_dir = (val & 0b0000_1000) >> 3; // bit 3
                self.channel1.env_pace = val & 0b0000_0111; // bits 2-0
            }
            0xFF13 => self.channel1.period = (self.channel1.period & 0b111_0000_0000) | val as u16,
            0xFF14 => {
                self.channel1.period =
                    (self.channel1.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);

                if val & 0b1000_0000 != 0 {
                    self.channel1.enabled = true;
                    self.channel1.freq_timer = (2048 - self.channel1.period as u32) * 4;
                    self.channel1.volume = self.channel1.initial_volume; // reset volume
                }
            }

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
