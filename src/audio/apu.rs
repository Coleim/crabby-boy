use std::sync::{Arc, Mutex};

use crate::audio::{audio_buffer::AudioBuffer, channel::Channel};

pub struct APU {
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    cycle_count: u32,
    tick_counter: f64,
    tick_per_sample: f64,
    is_on: bool,
    channel1: Channel,
    channel2: Channel,
    nr50: u8,
    nr51: u8,
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
            tick_per_sample: 0.0,
            is_on: false,
            channel1: Channel::default(),
            channel2: Channel::default(),
            nr50: 0,
            nr51: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.tick_per_sample = APU::CPU_FREQ as f64 / sample_rate.max(1) as f64;
    }

    pub fn tick(&mut self) {
        if !self.is_on {
            return;
        }
        for _ in 0..4 {
            self.cycle_count += 1;

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

            self.tick_counter += 1.0; // accumulateur downsampling
            if self.tick_counter >= self.tick_per_sample {
                self.tick_counter -= self.tick_per_sample;
                let sample = self.get_sample();
                if let Some(buffer) = &self.audio_buffer {
                    buffer.lock().unwrap().push(sample);
                }
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
        if ch.sweep_pace == 0 {
            return;
        }

        ch.sweep_timer = ch.sweep_timer.saturating_sub(1);
        if ch.sweep_timer == 0 {
            ch.sweep_timer = ch.sweep_pace;
            // compute new period
            let delta = ch.period >> ch.sweep_step;
            if ch.sweep_addition {
                ch.period += delta;
            } else {
                ch.period = ch.period.saturating_sub(delta);
            }
            if ch.period > 0b111_1111_1111 {
                ch.enabled = false;
            }
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
        self.channel1.freq_timer = self.channel1.freq_timer.saturating_sub(1);
        if self.channel1.freq_timer == 0 {
            self.channel1.freq_timer = (2048 - self.channel1.period as u32) * 4;
            self.channel1.duty_pos = (self.channel1.duty_pos + 1) % 8;
        }
    }

    fn tick_channel2(&mut self) {
        self.channel2.freq_timer = self.channel2.freq_timer.saturating_sub(1);
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
            0xFF10 => {
                let mut status = 0b1000_0000;
                if !self.channel1.sweep_addition {
                    status |= 0b0000_1000;
                }
                status |= self.channel1.sweep_step & 0b0000_0111;
                status |= self.channel1.sweep_pace << 4;

                status
            }
            0xFF14 => {
                let mut status = 0b0011_1111;
                if self.channel1.enabled {
                    status |= 0b1000_0000;
                }
                if self.channel1.length_enabled {
                    status |= 0b0100_0000;
                }
                status
            }
            0xFF19 => {
                let mut status = 0b0011_1111;
                if self.channel2.enabled {
                    status |= 0b1000_0000;
                }
                if self.channel2.length_enabled {
                    status |= 0b0100_0000;
                }
                status
            }

            0xFF25 => self.nr51,
            0xFF24 => self.nr50,
            _ => {
                println!("[AUDIO REG] READ NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                0xFF
                // std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF26 => {
                self.is_on = (val & 0b1000_0000) != 0;
                if !self.is_on {
                    self.channel1.enabled = false;
                    self.channel2.enabled = false;
                }
            }
            // Channel 1
            0xFF10 => self.channel1.write_sweep(val),
            0xFF11 => self.channel1.write_nr1(val),
            0xFF12 => self.channel1.write_nr2(val),
            0xFF13 => self.channel1.write_nr3(val),
            0xFF14 => self.channel1.write_nr4(val),
            // Channel 2
            0xFF16 => self.channel2.write_nr1(val),
            0xFF17 => self.channel2.write_nr2(val),
            0xFF18 => self.channel2.write_nr3(val),
            0xFF19 => self.channel2.write_nr4(val),
            0xFF25 => self.nr51 = val,
            0xFF24 => self.nr50 = val,
            _ => {
                // println!("[AUDIO REG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                // std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }
}
