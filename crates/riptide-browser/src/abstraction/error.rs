//! Error types for browser abstraction

use thiserror::Error;

pub type AbstractionResult<T> = Result<T, AbstractionError>;

#[derive(Debug, Error)]
pub enum AbstractionError {
    #[error("Failed to create page: {0}")]
    PageCreation(String),
    #[error("Failed to navigate: {0}")]
    Navigation(String),
    #[error("Failed to retrieve content: {0}")]
    ContentRetrieval(String),
    #[error("Failed to evaluate script: {0}")]
    Evaluation(String),
    #[error("Failed to take screenshot: {0}")]
    Screenshot(String),
    #[error("Failed to generate PDF: {0}")]
    PdfGeneration(String),
    #[error("Failed to close page: {0}")]
    PageClose(String),
    #[error("Failed to close browser: {0}")]
    BrowserClose(String),
    #[error("Operation not supported: {0}")]
    Unsupported(String),
    #[error("{0}")]
    Other(String),
}
