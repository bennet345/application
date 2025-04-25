pub struct FramesPerSecond {
    time: u128,
    frames: u8,
    samples: u8,
    last: std::time::SystemTime,
}

impl FramesPerSecond {
    pub fn new(samples: u8) -> Self {
        Self {
            time: 0,
            frames: 0,
            samples,
            last: std::time::SystemTime::now(),
        }
    }

    pub fn sample(&mut self) -> Option<f64> {
        self.time += self.last.elapsed().unwrap().as_micros();
        self.last = std::time::SystemTime::now();
        self.frames += 1;
        if self.frames == self.samples {
            self.frames = 0;
            let output = 1_000_000.0 / self.time as f64 * self.samples as f64;
            self.time = 0;
            return Some(output);
        }
        None
    }
}
