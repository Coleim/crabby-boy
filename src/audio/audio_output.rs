use cpal::{
    BufferSize, SampleFormat, StreamConfig, SupportedBufferSize, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

use crate::audio::audio_buffer::AudioBuffer;

pub struct AudioOutput {
    _stream: cpal::Stream, // doit rester vivant sinon l'audio s'arrête
}

// **Doc cpal :** https://docs.rs/cpal/latest/cpal/
impl AudioOutput {
    pub fn new(buffer: Arc<Mutex<AudioBuffer>>) -> Option<(Self, u32)> {
        let host = default_host();
        let device = host.default_output_device()?;

        let mut found_f32 = false;
        let mut selected_channels: u16 = 2;
        let mut selected_sample_rate: u32 = 44_100;
        let mut selected_buffer_size = BufferSize::Default;

        for cfg in device.supported_output_configs().ok()? {
            if cfg.sample_format() != SampleFormat::F32 {
                continue;
            }
            selected_channels = cfg.channels();
            let min = cfg.min_sample_rate();
            let max = cfg.max_sample_rate();

            if !(min..=max).contains(&44_100) {
                selected_sample_rate = min;
            }
            if let SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
                let target = 2048;
                selected_buffer_size = BufferSize::Fixed(target.clamp(*min, *max));
            }
            found_f32 = true;
            break;
        }
        if !found_f32 {
            return None;
        }

        let config = StreamConfig {
            channels: selected_channels,
            sample_rate: selected_sample_rate,
            buffer_size: selected_buffer_size,
        };

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let mut last_sample = 0.0_f32;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // let mut last_sample: f32 = 0.0;
                    // let mut hp_x1 = 0.0; // memorise l'entree precedente
                    // let mut hp_y1 = 0.0; // memorise la sortie precedente
                    // let hp_a = 1.0; // coefficient du filtre
                    let channels = selected_channels as usize;

                    if let Ok(mut buf) = buffer.try_lock() {
                        for frame in data.chunks_mut(channels) {
                            if !buf.empty() {
                                last_sample = buf.pop();
                            }
                            for out in frame.iter_mut() {
                                *out = last_sample;
                            }
                        }
                    } else {
                        for out in data.iter_mut() {
                            *out = last_sample;
                        }
                    }
                },
                err_fn,
                None,
            )
            .ok()?;

        stream.play().ok()?;
        Some((AudioOutput { _stream: stream }, config.sample_rate))
    }
}
