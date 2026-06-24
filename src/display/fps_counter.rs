use std::time::{Duration, Instant};

pub struct FpsCounter {
    last_instant: Instant,
    frames: u32,
    last_fps: f32,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last_instant: Instant::now(),
            frames: 0,
            last_fps: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.frames += 1;

        let now = Instant::now();
        let elapsed = now - self.last_instant;

        if elapsed >= Duration::from_secs(1) {
            self.last_fps = self.frames as f32 / elapsed.as_secs_f32();
            self.frames = 0;
            self.last_instant = now;
        }
    }

    pub fn fps(&self) -> f32 {
        self.last_fps
    }
}
