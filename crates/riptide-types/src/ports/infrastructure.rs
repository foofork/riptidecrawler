//! Infrastructure ports for system-level concerns
//!
//! This module provides backend-agnostic interfaces for infrastructure:
//! - Clock abstraction for deterministic time in tests
//! - Entropy source for deterministic randomness in tests
//! - Re-exports cache storage from Phase 0
//!
//! # Design Goals
//!
//! - **Determinism**: Enable reproducible tests with fake implementations
//! - **Testability**: Control time and randomness in tests
//! - **Simplicity**: Minimal interfaces focused on common use cases
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{Clock, Entropy};
//! use chrono::Utc;
//! use std::time::SystemTime;
//!
//! async fn example(clock: &dyn Clock, entropy: &dyn Entropy) {
//!     // Get current time (controllable in tests)
//!     let now = clock.now();
//!     let utc_now = clock.now_utc();
//!
//!     // Generate random data (deterministic in tests)
//!     let random_bytes = entropy.random_bytes(16);
//!     let random_id = entropy.random_id();
//! }
//! ```

use chrono::{DateTime, Utc};
use std::time::SystemTime;

// Re-export cache storage from Phase 0
pub use super::cache::CacheStorage;

/// System clock abstraction
///
/// Provides time access that can be mocked for testing.
/// Production implementations return real system time,
/// test implementations can return fixed or controllable time.
pub trait Clock: Send + Sync {
    /// Get current system time
    ///
    /// # Returns
    ///
    /// Current time as `SystemTime`
    fn now(&self) -> SystemTime;

    /// Get current UTC time
    ///
    /// # Returns
    ///
    /// Current time as `DateTime<Utc>`
    fn now_utc(&self) -> DateTime<Utc>;

    /// Get current Unix timestamp in seconds
    ///
    /// # Returns
    ///
    /// Seconds since Unix epoch
    fn timestamp(&self) -> u64 {
        self.now_utc().timestamp() as u64
    }

    /// Get current Unix timestamp in milliseconds
    ///
    /// # Returns
    ///
    /// Milliseconds since Unix epoch
    fn timestamp_millis(&self) -> u64 {
        self.now_utc().timestamp_millis() as u64
    }
}

/// Entropy source abstraction
///
/// Provides randomness that can be made deterministic for testing.
/// Production implementations use cryptographically secure randomness,
/// test implementations can use seeded pseudo-random generators.
pub trait Entropy: Send + Sync {
    /// Generate random bytes
    ///
    /// # Arguments
    ///
    /// * `len` - Number of random bytes to generate
    ///
    /// # Returns
    ///
    /// Vector of random bytes
    ///
    /// # Security
    ///
    /// Production implementations should use cryptographically
    /// secure random number generators (CSPRNG).
    fn random_bytes(&self, len: usize) -> Vec<u8>;

    /// Generate random identifier
    ///
    /// # Returns
    ///
    /// Random identifier string (e.g., UUID, nanoid)
    ///
    /// # Format
    ///
    /// Default format is UUID v4. Implementations may
    /// provide different formats.
    fn random_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate random integer in range
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum value (inclusive)
    /// * `max` - Maximum value (exclusive)
    ///
    /// # Returns
    ///
    /// Random integer in [min, max)
    fn random_range(&self, min: u64, max: u64) -> u64 {
        if max <= min {
            return min;
        }
        let range = max - min;
        let bytes = self.random_bytes(8);
        let random = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        min + (random % range)
    }

    /// Generate random alphanumeric string
    ///
    /// # Arguments
    ///
    /// * `len` - Length of string to generate
    ///
    /// # Returns
    ///
    /// Random alphanumeric string (A-Z, a-z, 0-9)
    fn random_string(&self, len: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let bytes = self.random_bytes(len);
        bytes
            .iter()
            .map(|&b| CHARS[(b as usize) % CHARS.len()] as char)
            .collect()
    }
}

// ============================================================================
// Production Implementations
// ============================================================================

/// System clock implementation using real system time
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// System entropy implementation using cryptographically secure RNG
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemEntropy;

impl Entropy for SystemEntropy {
    fn random_bytes(&self, len: usize) -> Vec<u8> {
        use uuid::Uuid;

        // For larger byte arrays, we generate multiple UUIDs
        let mut bytes = Vec::with_capacity(len);
        while bytes.len() < len {
            let uuid = Uuid::new_v4();
            let uuid_bytes = uuid.as_bytes();
            let remaining = len - bytes.len();
            let to_copy = remaining.min(uuid_bytes.len());
            bytes.extend_from_slice(&uuid_bytes[..to_copy]);
        }
        bytes.truncate(len);
        bytes
    }
}

// ============================================================================
// Test Implementations
// ============================================================================

/// Fake clock for deterministic testing
///
/// Allows controlling time in tests. Time doesn't advance
/// unless explicitly set.
#[derive(Debug, Clone)]
pub struct FakeClock {
    time: std::sync::Arc<std::sync::Mutex<DateTime<Utc>>>,
}

impl FakeClock {
    /// Create new fake clock with specified time
    pub fn new(time: DateTime<Utc>) -> Self {
        Self {
            time: std::sync::Arc::new(std::sync::Mutex::new(time)),
        }
    }

    /// Create fake clock at Unix epoch
    pub fn at_epoch() -> Self {
        Self::new(DateTime::from_timestamp(0, 0).unwrap())
    }

    /// Set clock to specific time
    pub fn set_time(&self, time: DateTime<Utc>) {
        *self.time.lock().unwrap() = time;
    }

    /// Advance clock by duration
    pub fn advance(&self, duration: std::time::Duration) {
        let mut time = self.time.lock().unwrap();
        *time += chrono::Duration::from_std(duration).unwrap();
    }
}

impl Default for FakeClock {
    fn default() -> Self {
        Self::at_epoch()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> SystemTime {
        let time = *self.time.lock().unwrap();
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(time.timestamp() as u64)
    }

    fn now_utc(&self) -> DateTime<Utc> {
        *self.time.lock().unwrap()
    }
}

/// Deterministic entropy for reproducible tests
///
/// Uses seeded pseudo-random generator for predictable randomness.
#[derive(Debug, Clone)]
pub struct DeterministicEntropy {
    seed: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl DeterministicEntropy {
    /// Create new deterministic entropy with seed
    pub fn new(seed: u64) -> Self {
        Self {
            seed: std::sync::Arc::new(std::sync::Mutex::new(seed)),
        }
    }

    /// Create with default seed (0)
    pub fn default_seed() -> Self {
        Self::new(0)
    }

    // Simple LCG random number generator
    fn next_u64(&self) -> u64 {
        let mut seed = self.seed.lock().unwrap();
        // Linear congruential generator parameters (from Numerical Recipes)
        const A: u64 = 1664525;
        const C: u64 = 1013904223;
        *seed = seed.wrapping_mul(A).wrapping_add(C);
        *seed
    }
}

impl Default for DeterministicEntropy {
    fn default() -> Self {
        Self::default_seed()
    }
}

impl Entropy for DeterministicEntropy {
    fn random_bytes(&self, len: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push((self.next_u64() & 0xFF) as u8);
        }
        bytes
    }

    fn random_id(&self) -> String {
        format!("deterministic-{:016x}", self.next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock() {
        let clock = SystemClock;
        let now = clock.now();
        let utc = clock.now_utc();

        assert!(now > SystemTime::UNIX_EPOCH);
        assert!(utc.timestamp() > 0);
    }

    #[test]
    fn test_system_entropy() {
        let entropy = SystemEntropy;
        let bytes1 = entropy.random_bytes(16);
        let bytes2 = entropy.random_bytes(16);

        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_fake_clock() {
        let clock = FakeClock::at_epoch();
        assert_eq!(clock.timestamp(), 0);

        clock.set_time(DateTime::from_timestamp(100, 0).unwrap());
        assert_eq!(clock.timestamp(), 100);

        clock.advance(std::time::Duration::from_secs(50));
        assert_eq!(clock.timestamp(), 150);
    }

    #[test]
    fn test_deterministic_entropy() {
        let entropy1 = DeterministicEntropy::new(42);
        let entropy2 = DeterministicEntropy::new(42);

        let bytes1 = entropy1.random_bytes(16);
        let bytes2 = entropy2.random_bytes(16);

        // Same seed should produce same bytes
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_entropy_random_range() {
        let entropy = DeterministicEntropy::new(42);

        for _ in 0..100 {
            let val = entropy.random_range(10, 20);
            assert!(val >= 10 && val < 20);
        }
    }

    #[test]
    fn test_entropy_random_string() {
        let entropy = DeterministicEntropy::new(42);
        let s = entropy.random_string(10);

        assert_eq!(s.len(), 10);
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }
}
