use std::time::{Duration, Instant};

/// Limits frames per second of the emulator to 60.
pub struct FpsLimiter {
    /// The time the last frame occured.
    last_frame: Instant,

    /// The time the next frame should occur.
    next_frame: Instant,
}

impl FpsLimiter {
    /// Create a new `FpsLimiter` instance.
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            last_frame: now,
            next_frame: now + Duration::from_secs_f64(1.0 / 59.73),
        }
    }

    /// Update the frame times, and return delta.
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now - self.last_frame;

        self.next_frame += Duration::from_secs_f64(1.0 / 59.73);
        self.last_frame = now;

        delta
    }

    /// Limit the FPS by sleeping till targetted next frame time.
    pub fn limit(&mut self) {
        let now = Instant::now();

        if now < self.next_frame {
            std::thread::sleep(self.next_frame - now);
        }
    }
}
