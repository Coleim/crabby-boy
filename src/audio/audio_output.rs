use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use crate::audio::audio_buffer::AudioBuffer;

pub struct AudioOutput {
    // _stream: cpal::Stream, // doit rester vivant sinon l'audio s'arrête
}

impl AudioOutput {
    pub fn new(buffer: Arc<Mutex<AudioBuffer>>) -> Self {
        AudioOutput {}
        // 1. let host = cpal::default_host();
        // 2. let device = host.default_output_device().unwrap();
        // 3. Configurer : sample_rate = 44100, channels = 1, sample_format = F32
        // 4. Construire le stream :
        //    device.build_output_stream(
        //        &config,
        //        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        //            let mut buf = buffer.lock().unwrap();
        //            for sample in data.iter_mut() {
        //                *sample = buf.pop();
        //            }
        //        },
        //        |err| eprintln!("Audio error: {}", err),
        //        None,  // timeout
        //    )
        // 5. stream.play().unwrap();
        // 6. Retourner AudioOutput { _stream: stream }
    }
}
