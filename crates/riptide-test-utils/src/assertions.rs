//! Custom assertions for EventMesh tests

/// Assert that a string contains all expected substrings
#[macro_export]
macro_rules! assert_contains_all {
    ($haystack:expr, $($needle:expr),+ $(,)?) => {
        $(
            assert!(
                $haystack.contains($needle),
                "Expected '{}' to contain '{}', but it didn't",
                $haystack,
                $needle
            );
        )+
    };
}

/// Assert that a string does not contain any of the specified substrings
#[macro_export]
macro_rules! assert_contains_none {
    ($haystack:expr, $($needle:expr),+ $(,)?) => {
        $(
            assert!(
                !$haystack.contains($needle),
                "Expected '{}' to NOT contain '{}', but it did",
                $haystack,
                $needle
            );
        )+
    };
}

/// Assert that execution time is within expected bounds
#[macro_export]
macro_rules! assert_duration {
    ($duration:expr, < $max:expr) => {
        assert!(
            $duration < $max,
            "Expected duration {:?} to be less than {:?}",
            $duration,
            $max
        );
    };
    ($duration:expr, > $min:expr) => {
        assert!(
            $duration > $min,
            "Expected duration {:?} to be greater than {:?}",
            $duration,
            $min
        );
    };
}

/// Performance assertion utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Assert that a closure completes within a time limit
    pub fn assert_completes_within<F>(duration: Duration, f: F)
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        f();
        let elapsed = start.elapsed();
        assert!(
            elapsed <= duration,
            "Operation took {:?}, expected <= {:?}",
            elapsed,
            duration
        );
    }

    /// Async version of assert_completes_within
    pub async fn assert_completes_within_async<F, Fut>(duration: Duration, f: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let start = Instant::now();
        f().await;
        let elapsed = start.elapsed();
        assert!(
            elapsed <= duration,
            "Async operation took {:?}, expected <= {:?}",
            elapsed,
            duration
        );
    }
}

/// HTML content assertions
pub mod html {
    /// Assert that HTML contains specific tags
    pub fn assert_has_tag(html: &str, tag: &str) {
        let open_tag = format!("<{}", tag);
        assert!(
            html.contains(&open_tag),
            "Expected HTML to contain <{} tag, but it didn't",
            tag
        );
    }

    /// Assert that HTML does not contain script tags
    pub fn assert_no_scripts(html: &str) {
        assert!(
            !html.contains("<script"),
            "Expected HTML to not contain <script> tags, but it did"
        );
        assert!(
            !html.contains("<SCRIPT"),
            "Expected HTML to not contain <SCRIPT> tags, but it did"
        );
    }

    /// Assert that HTML is well-formed (basic checks)
    pub fn assert_well_formed(html: &str) {
        // Check for basic HTML structure
        assert!(
            html.contains("<html") || html.contains("<!DOCTYPE"),
            "Expected HTML to contain <html or <!DOCTYPE"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_contains_all() {
        let text = "The quick brown fox jumps over the lazy dog";
        assert_contains_all!(text, "quick", "fox", "dog");
    }

    #[test]
    fn test_assert_contains_none() {
        let text = "The quick brown fox";
        assert_contains_none!(text, "cat", "mouse", "elephant");
    }

    #[test]
    fn test_performance_assertion() {
        use std::time::Duration;
        performance::assert_completes_within(Duration::from_millis(100), || {
            // Fast operation
            let _x = 1 + 1;
        });
    }

    #[tokio::test]
    async fn test_async_performance_assertion() {
        use std::time::Duration;
        performance::assert_completes_within_async(Duration::from_millis(100), || async {
            // Fast async operation
            tokio::time::sleep(Duration::from_millis(10)).await;
        })
        .await;
    }

    #[test]
    fn test_html_assertions() {
        let html = "<html><body><p>Test</p></body></html>";
        html::assert_has_tag(html, "html");
        html::assert_has_tag(html, "body");
        html::assert_has_tag(html, "p");
        html::assert_no_scripts(html);
        html::assert_well_formed(html);
    }
}
