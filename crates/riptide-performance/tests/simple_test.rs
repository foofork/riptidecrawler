//! Simple test to verify compilation
//! This ensures basic test infrastructure works

#[tokio::test]
async fn test_compilation() {
    // If this compiles and runs, the basic setup is working
    assert!(true);
}

#[test]
fn test_sync_compilation() {
    // Synchronous test
    assert_eq!(2 + 2, 4);
}
