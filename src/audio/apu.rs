use std::sync::{Arc, Mutex};

use crate::audio::audio_buffer::AudioBuffer;

#[derive(Default)]
pub struct Channel {
    duty_cycle: u8,
    duty_pos: u8,
    // lenght
    length_timer: u8,
    length_enabled: bool,
    // enveloppe
    env_timer: u8,
    env_dir: u8,  // 0=down, 1=up
    env_pace: u8, // 0..7, vitesse de l'enveloppe

    period: u16,
    volume: u8, // volume courant (change pendant le jeu)
    initial_volume: u8,
    freq_timer: u32,
    enabled: bool,
}
impl Channel {
    fn write_nr1(&mut self, val: u8) {
        self.duty_cycle = (val & 0b1100_0000) >> 6;
        self.length_timer = 64 - (val & 0b0011_1111);
    }
    fn write_nr2(&mut self, val: u8) {
        self.initial_volume = (val & 0b1111_0000) >> 4; // bits 7-4
        self.env_dir = (val & 0b0000_1000) >> 3; // bit 3
        self.env_pace = val & 0b0000_0111; // bits 2-0
        let dac_on = self.initial_volume != 0 || self.env_dir != 0;
        if !dac_on {
            self.enabled = false;
        }
    }
    fn write_nr3(&mut self, val: u8) {
        self.period = (self.period & 0b111_0000_0000) | val as u16;
    }
    fn write_nr4(&mut self, val: u8) {
        self.period = (self.period & 0b000_1111_1111) | ((val as u16 & 0b111) << 8);
        self.length_enabled = val & 0b0100_0000 != 0;
        if val & 0b1000_0000 != 0 {
            self.enabled = true;
            if self.length_timer == 0 {
                self.length_timer = 64;
            }
            self.freq_timer = (2048 - self.period as u32) * 4;
            self.volume = self.initial_volume; // reset volume
            self.env_timer = self.env_pace;
        }
    }
}

pub struct APU {
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    cycle_count: u32,
    tick_counter: f64,
    ticks_per_sample: f64,
    is_on: bool,
    channel1: Channel,
    channel2: Channel,
}

impl APU {
    const CPU_FREQ: u32 = 4_194_304;

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
            ticks_per_sample: APU::CPU_FREQ as f64 / 44_100.0,
            is_on: false,
            channel1: Channel::default(),
            channel2: Channel::default(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.ticks_per_sample = APU::CPU_FREQ as f64 / sample_rate.max(1) as f64;
    }

    pub fn tick(&mut self) {
        if !self.is_on {
            return;
        }
        self.cycle_count += 4;

        // Frame Sequencer : tous les 8192 T-cycles, on clock les composants lents
        if self.cycle_count % 8192 == 0 {
            let step = ((self.cycle_count / 8192) % 8) as u8;
            match step {
                0 | 2 | 4 | 6 => self.clock_length_all(), // 256 Hz
                _ => {}
            }
            if step == 7 {
                self.clock_envelope_all(); // 64 Hz
            }
            if step == 2 || step == 6 {
                self.clock_sweep(); // 128 Hz
            }
        }

        self.tick_channel1();
        self.tick_channel2();
        // self.tick_channel3();
        // self.tick_channel4();

        self.tick_counter += 4.0; // accumulateur downsampling
        if self.tick_counter >= self.ticks_per_sample {
            self.tick_counter -= self.ticks_per_sample;
            let sample = self.get_sample();
            if let Some(buffer) = &self.audio_buffer {
                buffer.lock().unwrap().push(sample);
            }
        }
    }

    fn clock_envelope_all(&mut self) {
        for ch in [&mut self.channel1, &mut self.channel2] {
            if ch.env_pace == 0 {
                continue;
            }
            if ch.env_timer > 0 {
                ch.env_timer = ch.env_timer.saturating_sub(1);
            }
            if ch.env_timer == 0 {
                ch.env_timer = ch.env_pace;
                if ch.env_dir == 1 && ch.volume < 15 {
                    ch.volume += 1;
                }
                if ch.env_dir == 0 && ch.volume > 0 {
                    ch.volume -= 1;
                }
            }
        }
    }

    fn clock_sweep(&mut self) {
        let ch = &mut self.channel1;
        if ch.env_pace == 0 {
            return;
        }
    }

    fn clock_length_all(&mut self) {
        for ch in [&mut self.channel1, &mut self.channel2] {
            if ch.length_enabled && ch.length_timer > 0 {
                ch.length_timer = ch.length_timer.saturating_sub(1);
                if ch.length_timer == 0 {
                    ch.enabled = false;
                }
            }
        }
    }

    fn tick_channel1(&mut self) {
        self.channel1.freq_timer = self.channel1.freq_timer.saturating_sub(4);
        if self.channel1.freq_timer == 0 {
            self.channel1.freq_timer = (2048 - self.channel1.period as u32) * 4;
            self.channel1.duty_pos = (self.channel1.duty_pos + 1) % 8;
        }
    }

    fn tick_channel2(&mut self) {
        self.channel2.freq_timer = self.channel2.freq_timer.saturating_sub(4);
        if self.channel2.freq_timer == 0 {
            self.channel2.freq_timer = (2048 - self.channel2.period as u32) * 4;
            self.channel2.duty_pos = (self.channel2.duty_pos + 1) % 8;
        }
    }

    fn get_sample(&self) -> f32 {
        if !self.is_on {
            return 0.0;
        }

        let mut sample = 0.0;
        let mut channels_active = 0.0;

        for channel in [&self.channel1, &self.channel2] {
            if !channel.enabled {
                continue;
            }
            let duty = APU::DUTY_TABLE[channel.duty_cycle as usize][channel.duty_pos as usize];
            sample += duty as f32 * channel.volume as f32 / 15.0;
            channels_active += 1.0;
        }

        if channels_active == 0.0 {
            return 0.0;
        }

        (sample / channels_active).clamp(-1.0, 1.0)
    }

    pub fn set_audio_buffer(&mut self, buffer: Arc<Mutex<AudioBuffer>>) {
        self.audio_buffer = Some(buffer);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF26 => {
                let mut status = 0b0111_0000;
                if self.is_on {
                    status |= 0b1000_0000;
                }
                if self.channel1.enabled {
                    status |= 0b0000_0001;
                }
                if self.channel2.enabled {
                    status |= 0b0000_0010;
                }
                status
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF26 => {
                self.is_on = (val & 0b1000_0000) != 0;
            }
            // Channel 1
            0xFF11 => self.channel1.write_nr1(val),
            0xFF12 => self.channel1.write_nr2(val),
            0xFF13 => self.channel1.write_nr3(val),
            0xFF14 => self.channel1.write_nr4(val),
            // Channel 2
            0xFF16 => self.channel2.write_nr1(val),
            0xFF17 => self.channel2.write_nr2(val),
            0xFF18 => self.channel2.write_nr3(val),
            0xFF19 => self.channel2.write_nr4(val),
            _ => {
                println!("[AUDIO REG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                // std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }
}
