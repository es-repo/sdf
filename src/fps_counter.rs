use web_time::Instant;

pub struct FpsCounter {
    update_time: Instant,
    frames_since_update: u32,
    count: f64,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            update_time: Instant::now(),
            frames_since_update: 0,
            count: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.update_time = Instant::now();
        self.frames_since_update = 0;
        self.count = 0.0;
    }

    pub fn tick(&mut self) {
        self.frames_since_update += 1;
        let elapsed = self.update_time.elapsed().as_secs_f64();
        if elapsed >= 0.5 {
            self.count = self.frames_since_update as f64 / elapsed;
            self.update_time = Instant::now();
            self.frames_since_update = 0;
        }
    }

    pub fn count(&self) -> f64 {
        self.count
    }
}
