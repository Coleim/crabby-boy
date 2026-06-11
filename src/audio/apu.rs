use std::sync::{Arc, Mutex};

use crate::audio::{
    audio_buffer::AudioBuffer, channel::Channel, noise_channel::NoiseChannel,
    wave_channel::WaveChannel,
};

pub struct APU {
    audio_buffer: Option<Arc<Mutex<AudioBuffer>>>,
    cycle_count: u32,
    tick_counter: f64,
    tick_per_sample: f64,
    is_on: bool,
    channel1: Channel,
    channel2: Channel,
    channel3: WaveChannel,
    channel4: NoiseChannel,
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
            channel3: WaveChannel::default(),
            channel4: NoiseChannel::default(),
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

            self.channel1.tick();
            self.channel2.tick();
            self.channel3.tick();
            self.channel4.tick();

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

        let ch = &mut self.channel4;
        if ch.env_pace == 0 {
            return;
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
            if ch.sweep_substraction {
                ch.period = ch.period.saturating_sub(delta);
            } else {
                ch.period += delta;
                if ch.period > 0b111_1111_1111 {
                    ch.enabled = false;
                    return;
                }
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

        let ch3 = &mut self.channel3;
        if ch3.length_enabled && ch3.len_timer > 0 {
            ch3.len_timer = ch3.len_timer.saturating_sub(1);
            if ch3.len_timer == 0 {
                ch3.enabled = false;
            }
        }

        let ch4 = &mut self.channel4;
        if ch4.length_enabled && ch4.len_timer > 0 {
            ch4.len_timer = ch4.len_timer.saturating_sub(1);
            if ch4.len_timer == 0 {
                ch4.enabled = false;
            }
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

        // CH3
        if self.channel3.enabled && self.channel3.dac_enabled {
            let byte = self.channel3.wave_ram[(self.channel3.wave_index / 2) as usize];
            let nibble = if self.channel3.wave_index % 2 == 0 {
                (byte >> 4) & 0xF
            } else {
                byte & 0xF
            };
            let shifted = nibble
                >> match self.channel3.volume_level {
                    0 => 4,
                    1 => 0,
                    2 => 1,
                    3 => 2,
                    _ => 4,
                };
            sample += shifted as f32 / 15.0;
            channels_active += 1.0;
        }

        // CH4
        let ch4 = &self.channel4;
        if ch4.enabled {
            if ch4.lfsr & 0b0000_0001 != 0 {
                sample += ch4.volume as f32 / 15.0;
            }
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
                if self.channel3.enabled {
                    status |= 0b0000_0100;
                }
                if self.channel4.enabled {
                    status |= 0b0000_1000;
                }
                status
            }

            0xFF10 => {
                let mut status = 0b1000_0000;
                if self.channel1.sweep_substraction {
                    status |= 0b0000_1000;
                }
                status |= self.channel1.sweep_step & 0b0000_0111;
                status |= self.channel1.sweep_pace << 4;

                status
            }
            0xFF15 => 0xFF,

            0xFF11 => 0b0011_1111 | (self.channel1.duty_cycle << 6),
            0xFF16 => 0b0011_1111 | (self.channel2.duty_cycle << 6),

            0xFF12 => {
                self.channel1.env_pace
                    | (self.channel1.env_dir << 3)
                    | (self.channel1.initial_volume << 4)
            }
            0xFF17 => {
                self.channel2.env_pace
                    | (self.channel2.env_dir << 3)
                    | (self.channel2.initial_volume << 4)
            }

            0xFF13 => 0xFF,
            0xFF18 => 0xFF,

            0xFF14 => {
                let mut status = 0b1011_1111;
                if self.channel1.length_enabled {
                    status |= 0b0100_0000;
                }
                status
            }
            0xFF19 => {
                let mut status = 0b1011_1111;
                if self.channel2.length_enabled {
                    status |= 0b0100_0000;
                }
                status
            }

            0xFF25 => self.nr51,
            0xFF24 => self.nr50,

            // Channel 3
            0xFF1A => {
                let mut status = 0b0111_1111; // bits 6-0 always 1
                if self.channel3.dac_enabled {
                    status |= 0b1000_0000;
                }
                status
            }

            0xFF1B => 0xFF,
            0xFF1C => 0b1001_1111 | (self.channel3.volume_level << 5),
            0xFF1E => {
                let mut status = 0b1011_1111;
                if self.channel3.length_enabled {
                    status |= 0b0100_0000;
                }
                status
            }

            // Channel 4
            0xFF20 => 0xFF,
            0xFF21 => {
                self.channel4.env_pace
                    | (self.channel4.env_dir << 3)
                    | (self.channel4.initial_volume << 4)
            }
            0xFF22 => {
                (self.channel4.clock_shift << 4)
                    | ((self.channel4.short_mode as u8) << 3)
                    | (self.channel4.clock_divider & 0b0000_0111)
            }

            0xFF23 => {
                let mut status = 0b1011_1111;
                if self.channel4.length_enabled {
                    status |= 0b0100_0000;
                }
                status
            }

            0xFF30..=0xFF3F => self.channel3.read_wave_ram((addr - 0xFF30) as u8),
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
                    self.nr51 = 0;
                    self.nr50 = 0;
                    self.channel1.reset();
                    self.channel2.reset();
                    self.channel3.reset();
                    self.channel4.reset();
                }
            }

            _ if !self.is_on && !(0xFF30..=0xFF3F).contains(&addr) => {}

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
            // Channel 3
            0xFF1A => self.channel3.write_nr0(val),
            0xFF1B => self.channel3.write_nr1(val),
            0xFF1C => self.channel3.write_nr2(val),
            0xFF1D => self.channel3.write_nr3(val),
            0xFF1E => self.channel3.write_nr4(val),
            // FF30–FF3F — Wave pattern RAM
            0xFF30..=0xFF3F => self.channel3.write_wave_ram((addr - 0xFF30) as u8, val),
            // Channel 4
            0xFF20 => self.channel4.write_nr1(val),
            0xFF21 => self.channel4.write_nr2(val),
            0xFF22 => self.channel4.write_nr3(val),
            0xFF23 => self.channel4.write_nr4(val),
            _ => {
                // println!("[AUDIO REG] WRITE NOT IMPLEMENTED FOR ADDR: {:02X}", addr);
                // std::panic::panic_any("[AUDIO REG] Not implemented at the moment.");
            }
        }
    }
}
