//! Error type re-exports for convenience

pub use anyhow::{anyhow, bail, Context, Error as AnyhowError, Result as AnyhowResult};
pub use thiserror::Error;

/// Common result type using anyhow
pub type Result<T> = std::result::Result<T, AnyhowError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Error)]
    enum TestError {
        #[error("test error: {0}")]
        Test(String),
    }

    #[test]
    fn test_anyhow_error() {
        let err = anyhow!("test error");
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_thiserror_derive() {
        let err = TestError::Test("example".to_string());
        assert_eq!(err.to_string(), "test error: example");
    }

    #[test]
    fn test_result_type() {
        fn returns_ok() -> Result<i32> {
            Ok(42)
        }

        fn returns_err() -> Result<i32> {
            Err(anyhow!("error"))
        }

        assert_eq!(returns_ok().unwrap(), 42);
        assert!(returns_err().is_err());
    }

    #[test]
    fn test_context() {
        fn inner() -> Result<i32> {
            Err(anyhow!("inner error"))
        }

        fn outer() -> Result<i32> {
            inner().context("outer context")
        }

        let err = outer().unwrap_err();
        let err_string = err.to_string();
        assert!(err_string.contains("outer context"));
    }
}
