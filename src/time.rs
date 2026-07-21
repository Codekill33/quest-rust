use std::time::Duration;

#[cfg(not(feature = "wasm"))]
pub use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(feature = "wasm")]
#[derive(Clone, Copy, Debug)]
pub struct Instant {
    now: f64,
}

#[cfg(feature = "wasm")]
impl Instant {
    pub fn now() -> Self {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window.performance().expect("performance should be available");
        Self {
            now: performance.now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window.performance().expect("performance should be available");
        let diff = performance.now() - self.now;
        Duration::from_millis(diff as u64)
    }
}

#[cfg(feature = "wasm")]
impl std::ops::Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, rhs: Duration) -> Self::Output {
        Self {
            now: self.now - rhs.as_millis() as f64,
        }
    }
}

#[cfg(feature = "wasm")]
#[derive(Clone, Copy, Debug)]
pub struct SystemTime(f64);

#[cfg(feature = "wasm")]
impl SystemTime {
    pub fn now() -> Self {
        Self(js_sys::Date::now())
    }

    pub fn duration_since(&self, _earlier: SystemTime) -> Result<Duration, std::time::SystemTimeError> {
        let diff = self.0 - _earlier.0;
        if diff >= 0.0 {
            Ok(Duration::from_millis(diff as u64))
        } else {
            // Since SystemTimeError has private fields, we return an Ok(Duration::ZERO).
            // Callers that need duration_since(UNIX_EPOCH) will work correctly.
            Ok(Duration::ZERO)
        }
    }
}

#[cfg(feature = "wasm")]
pub const UNIX_EPOCH: SystemTime = SystemTime(0.0);
