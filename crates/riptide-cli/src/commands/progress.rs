//! Progress indication utilities for CLI commands
//!
//! Provides user-friendly progress indicators for long-running operations.
//!
//! **Note**: This is infrastructure code for Phase 5+ UI features.
//! Currently unused but designed for future CLI enhancements.

use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Simple progress indicator for CLI operations
/// Infrastructure: UI utility for future CLI enhancements
#[allow(dead_code)]
pub struct ProgressIndicator {
    message: String,
    start_time: Instant,
    spinner_chars: Vec<char>,
    current_spinner: usize,
    is_active: bool,
}

impl ProgressIndicator {
    /// Create a new progress indicator with a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            start_time: Instant::now(),
            spinner_chars: vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            current_spinner: 0,
            is_active: false,
        }
    }

    /// Start the progress indicator
    pub fn start(&mut self) {
        self.is_active = true;
        self.start_time = Instant::now();
        self.update();
    }

    /// Update the progress indicator (show next spinner frame)
    pub fn update(&mut self) {
        if !self.is_active {
            return;
        }

        let elapsed = self.start_time.elapsed();
        let spinner = self.spinner_chars[self.current_spinner];
        self.current_spinner = (self.current_spinner + 1) % self.spinner_chars.len();

        print!(
            "\r{} {} ({:.1}s)",
            spinner,
            self.message,
            elapsed.as_secs_f64()
        );
        io::stdout().flush().ok();
    }

    /// Update the message while keeping the spinner running
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.update();
    }

    /// Finish the progress indicator with success
    pub fn finish_success(&mut self, final_message: &str) {
        self.is_active = false;
        let elapsed = self.start_time.elapsed();
        println!("\r✅ {} ({:.1}s)", final_message, elapsed.as_secs_f64());
    }

    /// Finish the progress indicator with error
    pub fn finish_error(&mut self, error_message: &str) {
        self.is_active = false;
        let elapsed = self.start_time.elapsed();
        println!("\r❌ {} ({:.1}s)", error_message, elapsed.as_secs_f64());
    }

    /// Finish the progress indicator with warning
    pub fn finish_warning(&mut self, warning_message: &str) {
        self.is_active = false;
        let elapsed = self.start_time.elapsed();
        println!("\r⚠️  {} ({:.1}s)", warning_message, elapsed.as_secs_f64());
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Progress bar for operations with known total steps
/// Infrastructure: Progress bar utility for CLI
#[allow(dead_code)]
pub struct ProgressBar {
    total: usize,
    current: usize,
    message: String,
    start_time: Instant,
    width: usize,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: usize, message: impl Into<String>) -> Self {
        Self {
            total,
            current: 0,
            message: message.into(),
            start_time: Instant::now(),
            width: 40,
        }
    }

    /// Increment progress
    pub fn inc(&mut self, delta: usize) {
        self.current = (self.current + delta).min(self.total);
        self.render();
    }

    /// Set progress to specific value
    pub fn set(&mut self, current: usize) {
        self.current = current.min(self.total);
        self.render();
    }

    /// Render the progress bar
    fn render(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };

        let filled = ((self.current as f64 / self.total as f64) * self.width as f64) as usize;
        let empty = self.width.saturating_sub(filled);

        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

        let elapsed = self.start_time.elapsed();
        let eta = if self.current > 0 {
            let rate = self.current as f64 / elapsed.as_secs_f64();
            let remaining = self.total - self.current;
            Duration::from_secs_f64(remaining as f64 / rate)
        } else {
            Duration::from_secs(0)
        };

        print!(
            "\r{} {} {:.1}% ({}/{}) - ETA: {:.0}s  ",
            self.message,
            bar,
            percentage,
            self.current,
            self.total,
            eta.as_secs_f64()
        );
        io::stdout().flush().ok();
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        println!(
            "\r✅ {} - Completed {}/{} in {:.1}s{}",
            self.message,
            self.total,
            self.total,
            elapsed.as_secs_f64(),
            " ".repeat(20) // Clear any remaining characters
        );
    }
}

/// Multi-step progress tracker
/// Infrastructure: Multi-step workflow tracker
#[allow(dead_code)]
pub struct MultiStepProgress {
    steps: Vec<String>,
    current_step: usize,
    start_time: Instant,
}

impl MultiStepProgress {
    /// Create a new multi-step progress tracker
    pub fn new(steps: Vec<String>) -> Self {
        Self {
            steps,
            current_step: 0,
            start_time: Instant::now(),
        }
    }

    /// Start the next step
    pub fn next_step(&mut self) -> Option<&str> {
        if self.current_step < self.steps.len() {
            let step = &self.steps[self.current_step];
            println!(
                "\n[{}/{}] {}...",
                self.current_step + 1,
                self.steps.len(),
                step
            );
            self.current_step += 1;
            Some(step)
        } else {
            None
        }
    }

    /// Finish all steps
    pub fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        println!(
            "\n✅ All {} steps completed in {:.1}s",
            self.steps.len(),
            elapsed.as_secs_f64()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator_creation() {
        let progress = ProgressIndicator::new("Testing");
        assert_eq!(progress.message, "Testing");
        assert!(!progress.is_active);
    }

    #[test]
    fn test_progress_bar_percentage() {
        let mut bar = ProgressBar::new(100, "Test");
        bar.set(50);
        assert_eq!(bar.current, 50);
        assert_eq!(bar.total, 100);
    }

    #[test]
    fn test_multi_step_progress() {
        let mut progress = MultiStepProgress::new(vec![
            "Step 1".to_string(),
            "Step 2".to_string(),
            "Step 3".to_string(),
        ]);

        assert_eq!(progress.next_step(), Some("Step 1"));
        assert_eq!(progress.next_step(), Some("Step 2"));
        assert_eq!(progress.next_step(), Some("Step 3"));
        assert_eq!(progress.next_step(), None);
    }
}
