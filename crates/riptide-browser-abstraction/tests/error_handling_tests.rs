//! Comprehensive error handling tests for browser abstraction
//!
//! Tests all AbstractionError variants and error conversion paths

use riptide_browser_abstraction::{AbstractionError, AbstractionResult};

#[test]
fn test_page_creation_error() {
    let error = AbstractionError::PageCreation("failed to launch".to_string());
    assert_eq!(error.to_string(), "Failed to create page: failed to launch");
}

#[test]
fn test_navigation_error() {
    let error = AbstractionError::Navigation("timeout exceeded".to_string());
    assert_eq!(error.to_string(), "Failed to navigate: timeout exceeded");
}

#[test]
fn test_content_retrieval_error() {
    let error = AbstractionError::ContentRetrieval("DOM not ready".to_string());
    assert_eq!(
        error.to_string(),
        "Failed to retrieve content: DOM not ready"
    );
}

#[test]
fn test_evaluation_error() {
    let error = AbstractionError::Evaluation("syntax error in script".to_string());
    assert_eq!(
        error.to_string(),
        "Failed to evaluate script: syntax error in script"
    );
}

#[test]
fn test_screenshot_error() {
    let error = AbstractionError::Screenshot("invalid format".to_string());
    assert_eq!(
        error.to_string(),
        "Failed to take screenshot: invalid format"
    );
}

#[test]
fn test_pdf_generation_error() {
    let error = AbstractionError::PdfGeneration("print failed".to_string());
    assert_eq!(error.to_string(), "Failed to generate PDF: print failed");
}

#[test]
fn test_page_close_error() {
    let error = AbstractionError::PageClose("connection lost".to_string());
    assert_eq!(error.to_string(), "Failed to close page: connection lost");
}

#[test]
fn test_browser_close_error() {
    let error = AbstractionError::BrowserClose("cleanup failed".to_string());
    assert_eq!(error.to_string(), "Failed to close browser: cleanup failed");
}

#[test]
fn test_unsupported_error() {
    let error = AbstractionError::Unsupported("feature not available".to_string());
    assert_eq!(
        error.to_string(),
        "Operation not supported: feature not available"
    );
}

#[test]
fn test_other_error() {
    let error = AbstractionError::Other("unexpected error".to_string());
    assert_eq!(error.to_string(), "unexpected error");
}

#[test]
fn test_error_result_ok() {
    let result: AbstractionResult<String> = Ok("success".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_error_result_err() {
    let result: AbstractionResult<String> = Err(AbstractionError::Other("failed".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_error_debug_formatting() {
    let error = AbstractionError::Navigation("timeout".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Navigation"));
    assert!(debug_str.contains("timeout"));
}

#[test]
fn test_error_chain_with_map_err() {
    fn might_fail() -> AbstractionResult<i32> {
        Err(AbstractionError::Other("internal error".to_string()))
    }

    let result =
        might_fail().map_err(|e| AbstractionError::PageCreation(format!("wrapped: {}", e)));

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("wrapped"));
        assert!(e.to_string().contains("internal error"));
    }
}

#[test]
fn test_multiple_error_variants() {
    let errors = vec![
        AbstractionError::PageCreation("error1".to_string()),
        AbstractionError::Navigation("error2".to_string()),
        AbstractionError::ContentRetrieval("error3".to_string()),
        AbstractionError::Evaluation("error4".to_string()),
        AbstractionError::Screenshot("error5".to_string()),
        AbstractionError::PdfGeneration("error6".to_string()),
        AbstractionError::PageClose("error7".to_string()),
        AbstractionError::BrowserClose("error8".to_string()),
        AbstractionError::Unsupported("error9".to_string()),
        AbstractionError::Other("error10".to_string()),
    ];

    assert_eq!(errors.len(), 10);

    // Verify each error has correct message
    assert!(errors[0].to_string().contains("Failed to create page"));
    assert!(errors[1].to_string().contains("Failed to navigate"));
    assert!(errors[2].to_string().contains("Failed to retrieve content"));
    assert!(errors[3].to_string().contains("Failed to evaluate script"));
    assert!(errors[4].to_string().contains("Failed to take screenshot"));
    assert!(errors[5].to_string().contains("Failed to generate PDF"));
    assert!(errors[6].to_string().contains("Failed to close page"));
    assert!(errors[7].to_string().contains("Failed to close browser"));
    assert!(errors[8].to_string().contains("Operation not supported"));
    assert!(errors[9].to_string().contains("error10"));
}

#[test]
fn test_error_with_empty_message() {
    let error = AbstractionError::Other("".to_string());
    assert_eq!(error.to_string(), "");
}

#[test]
fn test_error_with_special_characters() {
    let error = AbstractionError::Navigation("Failed: <script>alert('xss')</script>".to_string());
    let msg = error.to_string();
    assert!(msg.contains("<script>"));
    assert!(msg.contains("alert"));
}

#[test]
fn test_error_with_unicode() {
    let error = AbstractionError::ContentRetrieval("失敗: Japanese error".to_string());
    let msg = error.to_string();
    assert!(msg.contains("失敗"));
    assert!(msg.contains("Japanese"));
}
