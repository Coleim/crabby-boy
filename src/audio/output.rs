use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

use crate::audio::buffer::AudioBuffer;

pub struct AudioOutput {
    _stream: cpal::Stream,
}

impl AudioOutput {
    pub fn new(buffer: Arc<Mutex<AudioBuffer>>) -> Option<(Self, u32)> {
        let host = default_host();
        let device = host.default_output_device()?;
        let default_config = device.default_output_config().ok()?;
        let sample_rate = default_config.sample_rate();
        let channels = default_config.channels() as usize;
        let config: cpal::StreamConfig = default_config.into();

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        let sample = if !buf.empty() { buf.pop() } else { 0.0 };
                        for out in frame.iter_mut() {
                            *out = sample;
                        }
                    }
                },
                err_fn,
                None,
            )
            .ok()?;

        stream.play().ok()?;
        Some((AudioOutput { _stream: stream }, sample_rate))
    }
}
