use std::time::Duration;
use crate::time::Instant;

#[derive(Debug, PartialEq, Clone)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Stopped,
}

#[derive(Debug)]
pub struct Timer {
    state: TimerState,
    accumulated: Duration,
    start_instant: Option<Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            state: TimerState::Idle,
            accumulated: Duration::ZERO,
            start_instant: None,
        }
    }

    pub fn start(&mut self) -> Result<(), &'static str> {
        match self.state {
            TimerState::Idle => {
                self.start_instant = Some(Instant::now());
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Running => Err("Timer is already running"),
            TimerState::Paused => Err("Timer is paused; use resume() instead"),
            TimerState::Stopped => Err("Timer is stopped; create a new Timer to start again"),
        }
    }

    pub fn pause(&mut self) -> Result<(), &'static str> {
        match self.state {
            TimerState::Running => {
                if let Some(instant) = self.start_instant.take() {
                    self.accumulated += instant.elapsed();
                }
                self.state = TimerState::Paused;
                Ok(())
            }
            TimerState::Idle => Err("Timer has not been started"),
            TimerState::Paused => Err("Timer is already paused"),
            TimerState::Stopped => Err("Timer is already stopped"),
        }
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        match self.state {
            TimerState::Paused => {
                self.start_instant = Some(Instant::now());
                self.state = TimerState::Running;
                Ok(())
            }
            TimerState::Idle => Err("Timer has not been started"),
            TimerState::Running => Err("Timer is already running"),
            TimerState::Stopped => Err("Timer is already stopped"),
        }
    }

    pub fn stop(&mut self) -> Result<u64, &'static str> {
        match self.state {
            TimerState::Running => {
                if let Some(instant) = self.start_instant.take() {
                    self.accumulated += instant.elapsed();
                }
                self.state = TimerState::Stopped;
                Ok(self.elapsed_ms())
            }
            TimerState::Paused => {
                self.state = TimerState::Stopped;
                Ok(self.elapsed_ms())
            }
            TimerState::Idle => Err("Timer has not been started"),
            TimerState::Stopped => Err("Timer is already stopped"),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        let running_delta = match self.state {
            TimerState::Running => self
                .start_instant
                .map(|i| i.elapsed())
                .unwrap_or(Duration::ZERO),
            _ => Duration::ZERO,
        };
        (self.accumulated + running_delta).as_millis() as u64
    }

    pub fn state(&self) -> &TimerState {
        &self.state
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_timer_is_idle() {
        let timer = Timer::new();
        assert_eq!(*timer.state(), TimerState::Idle);
        assert_eq!(timer.elapsed_ms(), 0);
    }

    #[test]
    fn test_start_transitions_to_running() {
        let mut timer = Timer::new();
        assert!(timer.start().is_ok());
        assert_eq!(*timer.state(), TimerState::Running);
    }

    #[test]
    fn test_start_already_running_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        assert!(timer.start().is_err());
    }

    #[test]
    fn test_start_stopped_timer_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.stop().unwrap();
        assert!(timer.start().is_err());
    }

    #[test]
    fn test_pause_transitions_to_paused() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        assert!(timer.pause().is_ok());
        assert_eq!(*timer.state(), TimerState::Paused);
    }

    #[test]
    fn test_pause_idle_timer_errors() {
        let mut timer = Timer::new();
        assert!(timer.pause().is_err());
    }

    #[test]
    fn test_pause_already_paused_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.pause().unwrap();
        assert!(timer.pause().is_err());
    }

    #[test]
    fn test_pause_stopped_timer_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.stop().unwrap();
        assert!(timer.pause().is_err());
    }

    #[test]
    fn test_resume_transitions_to_running() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.pause().unwrap();
        assert!(timer.resume().is_ok());
        assert_eq!(*timer.state(), TimerState::Running);
    }

    #[test]
    fn test_resume_idle_timer_errors() {
        let mut timer = Timer::new();
        assert!(timer.resume().is_err());
    }

    #[test]
    fn test_resume_running_timer_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        assert!(timer.resume().is_err());
    }

    #[test]
    fn test_resume_stopped_timer_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.stop().unwrap();
        assert!(timer.resume().is_err());
    }

    #[test]
    fn test_stop_from_running_returns_elapsed_ms() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.stop().unwrap();
        assert!(elapsed >= 10, "Expected at least 10ms, got {elapsed}ms");
        assert_eq!(*timer.state(), TimerState::Stopped);
    }

    #[test]
    fn test_stop_from_paused_returns_elapsed_ms() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        thread::sleep(std::time::Duration::from_millis(10));
        timer.pause().unwrap();
        let elapsed = timer.stop().unwrap();
        assert!(elapsed >= 10, "Expected at least 10ms, got {elapsed}ms");
        assert_eq!(*timer.state(), TimerState::Stopped);
    }

    #[test]
    fn test_stop_idle_timer_errors() {
        let mut timer = Timer::new();
        assert!(timer.stop().is_err());
    }

    #[test]
    fn test_stop_already_stopped_errors() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        timer.stop().unwrap();
        assert!(timer.stop().is_err());
    }

    #[test]
    fn test_elapsed_ms_does_not_advance_while_paused() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        thread::sleep(std::time::Duration::from_millis(20));
        timer.pause().unwrap();
        let snapshot = timer.elapsed_ms();
        thread::sleep(std::time::Duration::from_millis(30));
        assert_eq!(
            timer.elapsed_ms(),
            snapshot,
            "Elapsed should not change while paused"
        );
    }

    #[test]
    fn test_elapsed_ms_accumulates_across_pause_resume() {
        let mut timer = Timer::new();
        timer.start().unwrap();
        thread::sleep(std::time::Duration::from_millis(20));
        timer.pause().unwrap();
        thread::sleep(std::time::Duration::from_millis(50));
        timer.resume().unwrap();
        thread::sleep(std::time::Duration::from_millis(20));
        let elapsed = timer.stop().unwrap();
        // should be ~40ms (20+20), not 90ms
        assert!(elapsed >= 40, "Expected at least 40ms, got {elapsed}ms");
        assert!(elapsed < 90, "Paused time should not count; got {elapsed}ms");
    }

    #[test]
    fn test_elapsed_ms_idle_is_zero() {
        let timer = Timer::new();
        assert_eq!(timer.elapsed_ms(), 0);
    }
}
