//! Error types for Python bindings
//!
//! Converts Rust errors to Python exceptions.

use pyo3::exceptions::{PyException, PyRuntimeError, PyTimeoutError, PyValueError};
use pyo3::prelude::*;

/// Custom RipTide error type
///
/// This wraps various error types and converts them to appropriate
/// Python exceptions.
pub struct RipTideError {
    message: String,
    error_type: ErrorType,
}

#[derive(Debug, Clone, Copy)]
enum ErrorType {
    Value,
    Runtime,
    Timeout,
    Network,
    Extraction,
    Initialization,
}

impl RipTideError {
    pub fn value_error<S: Into<String>>(msg: S) -> PyErr {
        PyValueError::new_err(msg.into())
    }

    pub fn runtime_error<S: Into<String>>(msg: S) -> PyErr {
        PyRuntimeError::new_err(msg.into())
    }

    pub fn timeout_error<S: Into<String>>(msg: S) -> PyErr {
        PyTimeoutError::new_err(msg.into())
    }

    pub fn network_error<S: Into<String>>(msg: S) -> PyErr {
        PyRuntimeError::new_err(format!("Network error: {}", msg.into()))
    }

    pub fn extraction_error<S: Into<String>>(msg: S) -> PyErr {
        PyRuntimeError::new_err(format!("Extraction error: {}", msg.into()))
    }

    pub fn initialization_error<S: Into<String>>(msg: S) -> PyErr {
        PyRuntimeError::new_err(format!("Initialization error: {}", msg.into()))
    }
}

impl From<RipTideError> for PyErr {
    fn from(err: RipTideError) -> PyErr {
        match err.error_type {
            ErrorType::Value => PyValueError::new_err(err.message),
            ErrorType::Runtime => PyRuntimeError::new_err(err.message),
            ErrorType::Timeout => PyTimeoutError::new_err(err.message),
            ErrorType::Network => {
                PyRuntimeError::new_err(format!("Network error: {}", err.message))
            }
            ErrorType::Extraction => {
                PyRuntimeError::new_err(format!("Extraction error: {}", err.message))
            }
            ErrorType::Initialization => {
                PyRuntimeError::new_err(format!("Initialization error: {}", err.message))
            }
        }
    }
}
