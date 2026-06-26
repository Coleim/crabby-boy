mod audio;
mod bus;
mod cpu;
mod crabby_boy;
mod display;
mod hardware;

use crate::audio::audio_output::AudioOutput;
use crate::bus::bus::Bus;
use crate::crabby_boy::CrabbyBoy;
use crate::display::display::Display;
use crate::display::ratatui_display::RatatuiDisplay;
use std::env;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "./tests/Kirby.gb"
    };

    let mut crabby = CrabbyBoy::new(file_path)?;
    // let _audio_output =
    //     if let Some((output, sample_rate)) = AudioOutput::new(crabby.audio_buffer.clone()) {
    //         // Very important to play sound
    //         crabby.set_audio_sample_rate(sample_rate);
    //         eprintln!("Audio initialized at {} Hz", sample_rate);
    //         Some(output)
    //     } else {
    //         None
    //     };
    //
    let mut display = RatatuiDisplay::new();

    // We need to target 60 FPS
    let target_frame = Duration::from_micros(16_667); // ~60FPS
    let mut last_tick_instant = Instant::now();

    while display.is_running() {
        let frame_start = Instant::now();
        let dt = frame_start.duration_since(last_tick_instant);
        last_tick_instant = frame_start;

        display.handle_events();

        crabby.tick_for_duration(dt);

        display.draw(&crabby);

        let frame_elapsed = frame_start.elapsed();
        if frame_elapsed < target_frame {
            std::thread::sleep(target_frame - frame_elapsed);
        }
    }

    Ok(())
}
