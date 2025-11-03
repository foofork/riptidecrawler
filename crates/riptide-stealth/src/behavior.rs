//! Human-like behavior simulation for browser automation.
//!
//! This module provides realistic mouse movement, scrolling, and interaction
//! patterns that mimic human behavior to evade detection by anti-bot systems.
//!
//! Features:
//! - Cubic Bézier curve mouse movement
//! - Smooth scrolling with easing functions
//! - Random reading pauses
//! - Natural click patterns
//! - Human-like timing variations

use rand::{Rng, SeedableRng};
use std::time::Duration;

/// Human-like behavior simulator
///
/// Generates realistic interaction patterns including:
/// - Mouse movements along Bézier curves
/// - Smooth scrolling with easing
/// - Natural timing variations
/// - Reading pauses
pub struct BehaviorSimulator {
    /// Random number generator for variations (Send-safe)
    rng: rand::rngs::SmallRng,
}

/// A 2D point representing screen coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Mouse movement path with timing information
#[derive(Debug, Clone)]
pub struct MousePath {
    /// Points along the path
    pub points: Vec<Point>,
    /// Delay between each point in milliseconds
    pub delays: Vec<u64>,
}

/// Scroll action with smooth easing
#[derive(Debug, Clone)]
pub struct ScrollAction {
    /// Target scroll position (pixels from top)
    pub target_y: u32,
    /// Duration of the scroll in milliseconds
    pub duration_ms: u64,
    /// Reading pause after scroll in milliseconds
    pub pause_after_ms: u64,
}

impl Default for BehaviorSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorSimulator {
    /// Create a new behavior simulator
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::SmallRng::from_entropy(),
        }
    }

    /// Generate a human-like mouse movement path using Cubic Bézier curves
    ///
    /// Creates a smooth, natural-looking path from start to end point with:
    /// - Randomized control points for curve variation
    /// - Variable speed (faster in middle, slower at ends)
    /// - Realistic timing between points
    ///
    /// # Arguments
    /// * `start` - Starting point
    /// * `end` - Ending point
    /// * `steps` - Number of intermediate points (default: ~50-100 based on distance)
    pub fn generate_mouse_movement(
        &mut self,
        start: Point,
        end: Point,
        steps: Option<usize>,
    ) -> MousePath {
        let distance = start.distance_to(&end);

        // Calculate number of steps based on distance (more steps for longer distances)
        let num_steps = steps.unwrap_or_else(|| {
            // Distance is always positive, safe to convert to usize for small screen coordinates
            // Screen coordinates are typically < 10000px, so distance / 10 is well within usize range
            let base_steps_f64 = (distance / 10.0).round().clamp(30.0, 100.0);
            // Safe conversion: value is clamped to reasonable range (30-100), always positive
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let base_steps = base_steps_f64 as usize;
            base_steps.clamp(30, 100)
        });

        // Generate control points for Cubic Bézier curve
        let control1 = self.random_control_point(&start, &end, 0.33);
        let control2 = self.random_control_point(&start, &end, 0.67);

        let mut points = Vec::with_capacity(num_steps);
        let mut delays = Vec::with_capacity(num_steps);

        for i in 0..=num_steps {
            // Convert step index to normalized position (0.0 to 1.0)
            // Note: usize to f64 conversion loses precision beyond 2^53, but num_steps is always small (< 1000)
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f64) / (num_steps as f64);

            // Calculate point on Cubic Bézier curve
            let point = self.cubic_bezier(t, &start, &control1, &control2, &end);
            points.push(point);

            // Variable delay: slower at start/end, faster in middle
            let speed_factor = self.ease_in_out_cubic(t);
            let base_delay = 10.0; // Base delay in ms
            let delay_f64 = (base_delay * (1.0 + (1.0 - speed_factor))).round();

            // Add random jitter (±30%)
            let jitter = self.rng.gen_range(0.7..=1.3);
            // Note: u64::MAX as f64 loses precision, but we only care about reasonable timing values
            #[allow(clippy::cast_precision_loss)]
            let u64_max_f64 = u64::MAX as f64;
            let final_delay_f64 = (delay_f64 * jitter).round().clamp(5.0, u64_max_f64);

            // Safe conversion: values are clamped to reasonable delay range, rounded, always positive
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let final_delay = final_delay_f64 as u64;

            delays.push(final_delay.max(5)); // Minimum 5ms delay
        }

        MousePath { points, delays }
    }

    /// Generate a random control point for Bézier curve
    ///
    /// Creates control points that add natural curvature to the path
    fn random_control_point(&mut self, start: &Point, end: &Point, position: f64) -> Point {
        let base_x = start.x + (end.x - start.x) * position;
        let base_y = start.y + (end.y - start.y) * position;

        // Add perpendicular offset for curve variation
        let distance = start.distance_to(end);
        let max_offset = (distance * 0.2).min(100.0); // Max 20% of distance or 100px

        let offset = self.rng.gen_range(-max_offset..=max_offset);

        // Calculate perpendicular direction
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            let perp_x = -dy / length;
            let perp_y = dx / length;

            Point::new(base_x + perp_x * offset, base_y + perp_y * offset)
        } else {
            Point::new(base_x, base_y)
        }
    }

    /// Calculate point on Cubic Bézier curve
    ///
    /// B(t) = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
    fn cubic_bezier(&self, t: f64, p0: &Point, p1: &Point, p2: &Point, p3: &Point) -> Point {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        Point::new(
            mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
            mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
        )
    }

    /// Cubic ease-in-out function for smooth acceleration/deceleration
    ///
    /// Returns a value between 0.0 and 1.0 with smooth transitions
    fn ease_in_out_cubic(&self, t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            let f = 2.0 * t - 2.0;
            1.0 + f * f * f / 2.0
        }
    }

    /// Generate smooth scrolling action with human-like behavior
    ///
    /// Creates a scroll action with:
    /// - Smooth easing (not instant jumps)
    /// - Random reading pause after scroll
    /// - Realistic scroll duration based on distance
    ///
    /// # Arguments
    /// * `current_y` - Current scroll position
    /// * `target_y` - Target scroll position
    pub fn generate_scroll_action(&mut self, current_y: u32, target_y: u32) -> ScrollAction {
        let distance = target_y.abs_diff(current_y);

        // Duration scales with distance (but not linearly)
        // Short scrolls: 200-400ms
        // Long scrolls: 500-1000ms
        // Note: u32 to f64 conversion loses precision beyond 2^53, but scroll distances are < 100000px
        #[allow(clippy::cast_precision_loss)]
        let base_duration = 200.0 + (distance as f64).sqrt() * 10.0;
        let duration_f64 = base_duration.clamp(200.0, 1000.0);

        // Add random jitter to duration (±20%)
        let jitter = self.rng.gen_range(0.8..=1.2);
        #[allow(clippy::cast_precision_loss)]
        let u64_max_f64 = u64::MAX as f64;
        let final_duration_f64 = (duration_f64 * jitter).round().clamp(0.0, u64_max_f64);

        // Safe conversion: duration is always in reasonable millisecond range, rounded, always positive
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_duration = final_duration_f64 as u64;

        // Random reading pause after scroll (500ms to 3s)
        let pause_after_ms = self.rng.gen_range(500..=3000);

        ScrollAction {
            target_y,
            duration_ms: final_duration,
            pause_after_ms,
        }
    }

    /// Generate a reading pause duration
    ///
    /// Simulates human reading time with realistic distribution
    /// - Short pauses: 300-800ms (quick scan)
    /// - Medium pauses: 800-2000ms (reading)
    /// - Long pauses: 2000-5000ms (careful reading)
    pub fn generate_reading_pause(&mut self) -> Duration {
        // 60% short, 30% medium, 10% long
        let category = self.rng.gen_range(0..100);

        let ms = if category < 60 {
            // Short pause
            self.rng.gen_range(300..=800)
        } else if category < 90 {
            // Medium pause
            self.rng.gen_range(800..=2000)
        } else {
            // Long pause
            self.rng.gen_range(2000..=5000)
        };

        Duration::from_millis(ms)
    }

    /// Generate a random click delay
    ///
    /// Human click delays are typically 50-200ms after mouse arrives
    pub fn generate_click_delay(&mut self) -> Duration {
        let ms = self.rng.gen_range(50..=200);
        Duration::from_millis(ms)
    }

    /// Generate random typing delay between keystrokes
    ///
    /// Simulates realistic typing patterns:
    /// - Fast typers: 50-100ms
    /// - Average typers: 100-200ms
    /// - Slow typers: 200-400ms
    /// - Occasional pauses: 500-1000ms (thinking)
    pub fn generate_typing_delay(&mut self) -> Duration {
        // 10% chance of pause (thinking about what to type)
        if self.rng.gen_range(0..100) < 10 {
            let ms = self.rng.gen_range(500..=1000);
            return Duration::from_millis(ms);
        }

        // Normal typing speed distribution
        let ms = self.rng.gen_range(50..=300);
        Duration::from_millis(ms)
    }

    /// Generate page load wait time
    ///
    /// Simulates human waiting for page to load before interaction
    /// Typically 500-2000ms to allow for rendering and initial scan
    pub fn generate_page_load_wait(&mut self) -> Duration {
        let ms = self.rng.gen_range(500..=2000);
        Duration::from_millis(ms)
    }

    /// Generate random viewport offset for natural scrolling patterns
    ///
    /// Humans don't always scroll to exact positions - they overshoot
    /// or undershoot slightly
    pub fn add_scroll_offset(&mut self, target: u32, max_offset: u32) -> u32 {
        let offset = self.rng.gen_range(0..=max_offset);
        let direction = if self.rng.gen_bool(0.5) { 1i32 } else { -1i32 };

        // Calculate offset with safe arithmetic
        // u32 max is ~4 billion, scroll positions are always < i32::MAX
        let target_i32 = target.min(i32::MAX as u32) as i32;
        let offset_i32 = offset.min(i32::MAX as u32) as i32;

        let result = target_i32.saturating_add(offset_i32.saturating_mul(direction));
        // Result is clamped to non-negative values, safe to cast
        #[allow(clippy::cast_sign_loss)]
        let result_u32 = result.max(0) as u32;
        result_u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);

        assert_eq!(p1.distance_to(&p2), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_mouse_movement_generation() {
        let mut sim = BehaviorSimulator::new();
        let start = Point::new(100.0, 100.0);
        let end = Point::new(500.0, 300.0);

        let path = sim.generate_mouse_movement(start, end, None);

        // Should have points
        assert!(!path.points.is_empty());
        assert_eq!(path.points.len(), path.delays.len());

        // First point should be at start
        assert_eq!(path.points[0], start);

        // Last point should be at end
        let last = path.points.last().unwrap();
        assert!((last.x - end.x).abs() < 1.0);
        assert!((last.y - end.y).abs() < 1.0);

        // All delays should be reasonable (5-100ms)
        for delay in &path.delays {
            assert!(*delay >= 5 && *delay <= 100);
        }
    }

    #[test]
    fn test_bezier_curve_endpoints() {
        let sim = BehaviorSimulator::new();
        let p0 = Point::new(0.0, 0.0);
        let p1 = Point::new(100.0, 50.0);
        let p2 = Point::new(200.0, 150.0);
        let p3 = Point::new(300.0, 200.0);

        // At t=0, should be at start
        let start = sim.cubic_bezier(0.0, &p0, &p1, &p2, &p3);
        assert_eq!(start, p0);

        // At t=1, should be at end
        let end = sim.cubic_bezier(1.0, &p0, &p1, &p2, &p3);
        assert_eq!(end, p3);

        // At t=0.5, should be somewhere in between
        let mid = sim.cubic_bezier(0.5, &p0, &p1, &p2, &p3);
        assert!(mid.x > p0.x && mid.x < p3.x);
        assert!(mid.y > p0.y && mid.y < p3.y);
    }

    #[test]
    fn test_easing_function() {
        let sim = BehaviorSimulator::new();

        // At start and end, should be close to 0 and 1
        assert!((sim.ease_in_out_cubic(0.0) - 0.0).abs() < 0.01);
        assert!((sim.ease_in_out_cubic(1.0) - 1.0).abs() < 0.01);

        // Should be smooth (no discontinuities)
        let mid = sim.ease_in_out_cubic(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_scroll_action_generation() {
        let mut sim = BehaviorSimulator::new();

        let action = sim.generate_scroll_action(0, 1000);

        // Target should match
        assert_eq!(action.target_y, 1000);

        // Duration should be reasonable
        assert!(action.duration_ms >= 200 && action.duration_ms <= 1200);

        // Pause should be reasonable
        assert!(action.pause_after_ms >= 500 && action.pause_after_ms <= 3000);
    }

    #[test]
    fn test_reading_pause_distribution() {
        let mut sim = BehaviorSimulator::new();

        // Generate many pauses and check distribution
        let mut short_count = 0;
        let mut medium_count = 0;
        let mut long_count = 0;

        for _ in 0..1000 {
            let pause = sim.generate_reading_pause();
            let ms = pause.as_millis() as u64;

            if ms < 800 {
                short_count += 1;
            } else if ms < 2000 {
                medium_count += 1;
            } else {
                long_count += 1;
            }
        }

        // Rough distribution check (with tolerance for randomness)
        assert!(short_count > 500); // Should be ~60%
        assert!(medium_count > 200); // Should be ~30%
        assert!(long_count > 50); // Should be ~10%
    }

    #[test]
    fn test_click_delay_range() {
        let mut sim = BehaviorSimulator::new();

        for _ in 0..100 {
            let delay = sim.generate_click_delay();
            let ms = delay.as_millis() as u64;

            assert!((50..=200).contains(&ms));
        }
    }

    #[test]
    fn test_typing_delay_has_pauses() {
        let mut sim = BehaviorSimulator::new();

        let mut has_pause = false;
        let mut has_normal = false;

        // Generate many delays to ensure we hit both cases
        for _ in 0..200 {
            let delay = sim.generate_typing_delay();
            let ms = delay.as_millis() as u64;

            if ms >= 500 {
                has_pause = true;
            } else {
                has_normal = true;
            }

            if has_pause && has_normal {
                break;
            }
        }

        assert!(has_pause, "Should generate occasional pauses");
        assert!(has_normal, "Should generate normal typing delays");
    }

    #[test]
    fn test_page_load_wait() {
        let mut sim = BehaviorSimulator::new();

        for _ in 0..50 {
            let wait = sim.generate_page_load_wait();
            let ms = wait.as_millis() as u64;

            assert!((500..=2000).contains(&ms));
        }
    }

    #[test]
    fn test_scroll_offset() {
        let mut sim = BehaviorSimulator::new();
        let target = 1000u32;
        let max_offset = 50u32;

        for _ in 0..50 {
            let result = sim.add_scroll_offset(target, max_offset);

            // Should be within offset range
            let diff = result.abs_diff(target);

            assert!(diff <= max_offset);
        }
    }
}
