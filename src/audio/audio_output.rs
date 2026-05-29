use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

use crate::audio::audio_buffer::AudioBuffer;

pub struct AudioOutput {
    _stream: cpal::Stream, // doit rester vivant sinon l'audio s'arrête
}

// **Doc cpal :** https://docs.rs/cpal/latest/cpal/
impl AudioOutput {
    pub fn new(buffer: Arc<Mutex<AudioBuffer>>) -> Self {
        let host = default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

        // 3. Configurer : sample_rate = 44100, channels = 1, sample_format = F32
        let stream = device
            .build_output_stream(
                &supported_config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = if buf.empty() { buf.pop() } else { 0.0 }
                    }
                },
                // AudioOutput::write_silence::<f32>,
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        AudioOutput { _stream: stream }
    }
}
