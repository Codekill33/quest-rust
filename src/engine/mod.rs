use std::thread::sleep;
use std::time::Duration;
use crate::time::Instant;

/// Core game engine that manages the main loop and lifecycle.
pub struct Engine {
    tick_rate: Duration,
}

impl Engine {
    /// Create a new engine with the given tick rate (frame duration).
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    /// Initialize engine resources.
    pub fn init(&self) {
        println!("Engine: init");
    }

    /// Run the game loop for a specified duration. This enforces a fixed tick rate.
    pub fn run_for(&self, run_duration: Duration) {
        println!("Engine: run_for {:?}", run_duration);
        let start = Instant::now();
        let mut ticks: u64 = 0;
        while start.elapsed() < run_duration {
            let frame_start = Instant::now();

            // Placeholder for per-tick logic.
            ticks += 1;

            // Enforce fixed tick rate by sleeping the remainder of the frame.
            let frame_time = frame_start.elapsed();
            if frame_time < self.tick_rate {
                sleep(self.tick_rate - frame_time);
            }
        }
        println!("Engine: run complete (ticks={})", ticks);
    }

    /// Cleanly shutdown and release resources.
    pub fn shutdown(&self) {
        println!("Engine: shutdown");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn engine_runs_and_stops() {
        let engine = Engine::new(Duration::from_millis(16));
        engine.init();
        engine.run_for(Duration::from_millis(50));
        engine.shutdown();
    }
}
