use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

impl From<u8> for State {
    fn from(v: u8) -> Self {
        match v {
            1 => State::Open,
            2 => State::HalfOpen,
            _ => State::Closed,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub failure_threshold: u32,       // N failures → Open
    pub open_cooldown_ms: u64,        // time in Open
    pub half_open_max_in_flight: u32, // number of trial calls allowed
}

impl Default for Config {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_cooldown_ms: 30_000,
            half_open_max_in_flight: 3,
        }
    }
}

pub trait Clock: Send + Sync + std::fmt::Debug {
    fn now_ms(&self) -> u64;
}

#[derive(Default, Debug)]
pub struct RealClock;

impl Clock for RealClock {
    fn now_ms(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: AtomicU8,
    failures: AtomicU32,
    successes: AtomicU32,
    open_until_ms: AtomicU64,
    half_open_permits: Arc<Semaphore>,
    cfg: Config,
    clock: Arc<dyn Clock>,
}

impl CircuitBreaker {
    pub fn new(cfg: Config, clock: Arc<dyn Clock>) -> Arc<Self> {
        Arc::new(Self {
            state: AtomicU8::new(State::Closed as u8),
            failures: AtomicU32::new(0),
            successes: AtomicU32::new(0),
            open_until_ms: AtomicU64::new(0),
            half_open_permits: Arc::new(Semaphore::new(cfg.half_open_max_in_flight as usize)),
            cfg,
            clock,
        })
    }

    #[inline]
    pub fn state(&self) -> State {
        self.state.load(Relaxed).into()
    }

    /// Returns Ok(permit_guard) if allowed to proceed; Err if short-circuited
    pub fn try_acquire(&self) -> Result<Option<tokio::sync::OwnedSemaphorePermit>, &'static str> {
        match self.state() {
            State::Closed => Ok(None),
            State::Open => {
                let now = self.clock.now_ms();
                let open_until = self.open_until_ms.load(Relaxed);
                if now >= open_until {
                    // transition Open -> HalfOpen
                    self.state.store(State::HalfOpen as u8, Relaxed);
                } else {
                    return Err("circuit open");
                }
                // fallthrough to HalfOpen path
                self.try_acquire()
            }
            State::HalfOpen => match Arc::clone(&self.half_open_permits).try_acquire_owned() {
                Ok(permit) => Ok(Some(permit)),
                Err(_) => Err("half-open saturated"),
            },
        }
    }

    #[inline]
    pub fn on_success(&self) {
        match self.state() {
            State::Closed => {
                self.failures.store(0, Relaxed);
            }
            State::HalfOpen => {
                let succ = self.successes.fetch_add(1, Relaxed) + 1;
                if succ >= 1 {
                    // First success closes the circuit and fully resets
                    self.state.store(State::Closed as u8, Relaxed);
                    self.failures.store(0, Relaxed);
                    self.successes.store(0, Relaxed);
                    // refill semaphore (in case previous failures consumed)
                    let def = self.cfg.half_open_max_in_flight as usize;
                    let deficit = def.saturating_sub(self.half_open_permits.available_permits());
                    if deficit > 0 {
                        self.half_open_permits.add_permits(deficit);
                    }
                }
            }
            State::Open => {} // shouldn't happen; guarded by try_acquire
        }
    }

    #[inline]
    pub fn on_failure(&self) {
        match self.state() {
            State::Closed => {
                let f = self.failures.fetch_add(1, Relaxed) + 1;
                if f >= self.cfg.failure_threshold {
                    self.trip_open();
                }
            }
            State::HalfOpen => {
                // immediate reopen on any failure in half-open
                self.trip_open();
            }
            State::Open => {}
        }
    }

    #[inline]
    fn trip_open(&self) {
        self.state.store(State::Open as u8, Relaxed);
        self.successes.store(0, Relaxed);
        self.failures.store(0, Relaxed);
        let until = self.clock.now_ms() + self.cfg.open_cooldown_ms;
        self.open_until_ms.store(until, Relaxed);
        // reset half-open permits for the next time we enter HalfOpen
        let def = self.cfg.half_open_max_in_flight as usize;
        let avail = self.half_open_permits.available_permits();
        if avail < def {
            self.half_open_permits.add_permits(def - avail);
        }
    }

    pub fn failure_count(&self) -> u32 {
        self.failures.load(Relaxed)
    }
}

/// Helper function to wrap async calls with circuit breaker protection
pub async fn guarded_call<T, E, F, Fut>(cb: &Arc<CircuitBreaker>, f: F) -> Result<T, anyhow::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: Into<anyhow::Error>,
{
    let permit = cb
        .try_acquire()
        .map_err(|msg| anyhow::anyhow!("Circuit breaker rejected: {}", msg))?;

    let res = f().await;
    match &res {
        Ok(_) => cb.on_success(),
        Err(_) => cb.on_failure(),
    }
    drop(permit); // releases half-open permit if any
    res.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    #[derive(Default)]
    struct TestClock {
        now: AtomicU64,
    }

    impl TestClock {
        fn advance(&self, ms: u64) {
            self.now.fetch_add(ms, Relaxed);
        }

        fn set(&self, ms: u64) {
            self.now.store(ms, Relaxed);
        }
    }

    impl Clock for TestClock {
        fn now_ms(&self) -> u64 {
            self.now.load(Relaxed)
        }
    }

    #[test]
    fn circuit_transitions_closed_open_halfopen_closed() {
        let clock = Arc::new(TestClock::default());
        let cb = CircuitBreaker::new(
            Config {
                failure_threshold: 3,
                open_cooldown_ms: 5_000,
                half_open_max_in_flight: 2,
            },
            clock.clone(),
        );

        // Initial state is Closed
        assert_eq!(cb.state(), State::Closed);
        assert!(cb.try_acquire().unwrap().is_none());

        // 3 failures → Open
        cb.on_failure();
        assert_eq!(cb.state(), State::Closed);
        cb.on_failure();
        assert_eq!(cb.state(), State::Closed);
        cb.on_failure();
        assert_eq!(cb.state(), State::Open);

        // Should reject while cool-down not elapsed
        assert!(cb.try_acquire().is_err());

        // Advance time to exit Open
        clock.advance(5_000);

        // First acquire transitions to HalfOpen and grants a permit
        let permit = cb.try_acquire().expect("should get permit");
        assert!(permit.is_some());
        assert_eq!(cb.state(), State::HalfOpen);

        // Success closes
        cb.on_success();
        assert_eq!(cb.state(), State::Closed);
    }

    #[test]
    fn half_open_failure_reopens_immediately() {
        let clock = Arc::new(TestClock::default());
        let cb = CircuitBreaker::new(
            Config {
                failure_threshold: 2,
                open_cooldown_ms: 1_000,
                half_open_max_in_flight: 1,
            },
            clock.clone(),
        );

        // Trip to Open
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), State::Open);

        // Advance time and transition to HalfOpen
        clock.advance(1_000);
        let _permit = cb.try_acquire().expect("should get permit");
        assert_eq!(cb.state(), State::HalfOpen);

        // Failure in HalfOpen immediately reopens
        cb.on_failure();
        assert_eq!(cb.state(), State::Open);

        // Should need to wait again
        assert!(cb.try_acquire().is_err());

        // After another cooldown
        clock.advance(1_000);
        assert!(cb.try_acquire().is_ok());
    }

    #[test]
    fn half_open_respects_max_permits() {
        let clock = Arc::new(TestClock::default());
        let cb = CircuitBreaker::new(
            Config {
                failure_threshold: 1,
                open_cooldown_ms: 100,
                half_open_max_in_flight: 2,
            },
            clock.clone(),
        );

        // Trip to Open
        cb.on_failure();
        assert_eq!(cb.state(), State::Open);

        // Advance time and transition to HalfOpen
        clock.advance(100);

        // Should allow exactly 2 permits
        let p1 = cb.try_acquire().expect("first permit");
        assert!(p1.is_some());
        assert_eq!(cb.state(), State::HalfOpen);

        let p2 = cb.try_acquire().expect("second permit");
        assert!(p2.is_some());

        // Third should be rejected (saturated)
        assert!(cb.try_acquire().is_err());

        // Release one permit and try again
        drop(p1);

        // Now should get another permit after dropping
        let p3 = cb.try_acquire().expect("third permit after release");
        assert!(p3.is_some());
    }

    #[tokio::test(start_paused = true)]
    async fn circuit_breaker_with_tokio_time() {
        use std::time::Duration;

        let cb = CircuitBreaker::new(
            Config {
                failure_threshold: 3,
                open_cooldown_ms: 5_000,
                half_open_max_in_flight: 2,
            },
            Arc::new(RealClock),
        );

        // Note: with start_paused = true, we control time advancement
        assert_eq!(cb.state(), State::Closed);

        // Trip to Open
        cb.on_failure();
        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), State::Open);

        // Can't use try_acquire with RealClock in paused time directly,
        // but we demonstrate the pattern
        tokio::time::advance(Duration::from_millis(5_000)).await;

        // In real usage with RealClock, time has "passed"
        // This test shows the structure without hanging
    }
}
