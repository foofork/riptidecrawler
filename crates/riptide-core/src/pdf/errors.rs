use serde::{Deserialize, Serialize};

/// Errors that can occur during PDF processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PdfError {
    /// File is not a valid PDF
    InvalidPdf { message: String },

    /// PDF is encrypted and cannot be processed
    EncryptedPdf,

    /// PDF is too large to process
    FileTooLarge { size: u64, max_size: u64 },

    /// Corrupted or damaged PDF
    CorruptedPdf { message: String },

    /// Processing timeout
    Timeout { timeout_seconds: u64 },

    /// Memory limit exceeded
    MemoryLimit { used: u64, limit: u64 },

    /// Unsupported PDF version
    UnsupportedVersion { version: String },

    /// Internal processing error
    ProcessingError { message: String },

    /// IO error during processing
    IoError { message: String },
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfError::InvalidPdf { message } => write!(f, "Invalid PDF: {}", message),
            PdfError::EncryptedPdf => write!(f, "PDF is encrypted and cannot be processed"),
            PdfError::FileTooLarge { size, max_size } => {
                write!(
                    f,
                    "PDF file too large: {} bytes (max: {} bytes)",
                    size, max_size
                )
            }
            PdfError::CorruptedPdf { message } => write!(f, "Corrupted PDF: {}", message),
            PdfError::Timeout { timeout_seconds } => {
                write!(
                    f,
                    "PDF processing timeout after {} seconds",
                    timeout_seconds
                )
            }
            PdfError::MemoryLimit { used, limit } => {
                write!(
                    f,
                    "Memory limit exceeded: {} bytes used (limit: {} bytes)",
                    used, limit
                )
            }
            PdfError::UnsupportedVersion { version } => {
                write!(f, "Unsupported PDF version: {}", version)
            }
            PdfError::ProcessingError { message } => write!(f, "Processing error: {}", message),
            PdfError::IoError { message } => write!(f, "IO error: {}", message),
        }
    }
}

impl std::error::Error for PdfError {}

// Implement conversion from common error types
impl From<std::io::Error> for PdfError {
    fn from(error: std::io::Error) -> Self {
        PdfError::IoError {
            message: error.to_string(),
        }
    }
}

impl From<anyhow::Error> for PdfError {
    fn from(error: anyhow::Error) -> Self {
        PdfError::ProcessingError {
            message: error.to_string(),
        }
    }
}

/// Result type for PDF operations
pub type PdfResult<T> = Result<T, PdfError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = PdfError::InvalidPdf {
            message: "Not a PDF file".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid PDF: Not a PDF file");

        let error = PdfError::EncryptedPdf;
        assert_eq!(error.to_string(), "PDF is encrypted and cannot be processed");

        let error = PdfError::FileTooLarge {
            size: 1000000,
            max_size: 500000,
        };
        assert_eq!(
            error.to_string(),
            "PDF file too large: 1000000 bytes (max: 500000 bytes)"
        );
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let pdf_error = PdfError::from(io_error);

        match pdf_error {
            PdfError::IoError { message } => {
                assert!(message.contains("File not found"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_error_is_error_trait() {
        let error = PdfError::ProcessingError {
            message: "Test error".to_string(),
        };

        // This tests that PdfError implements the Error trait
        let _error_trait: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_pdf_result_type() {
        let success: PdfResult<String> = Ok("Success".to_string());
        assert!(success.is_ok());

        let failure: PdfResult<String> = Err(PdfError::EncryptedPdf);
        assert!(failure.is_err());
    }
}