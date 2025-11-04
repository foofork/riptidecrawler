//! Demonstration of secrets redaction in Debug output
//!
//! This example shows how sensitive data is automatically redacted in Debug output
//! to prevent accidental credential exposure in logs.
//!
//! Run with: cargo run --example secrets_demo -p riptide-types

use riptide_types::secrets::SecretString;

fn main() {
    println!("=== Secrets Redaction Demo ===\n");

    // Create a secret API key
    let api_key = SecretString::new("sk_live_production_api_key_abcdef1234567890".to_string());

    println!("1. Secret API Key:");
    println!("   Debug output: {:?}", api_key);
    println!("   ✓ Notice only first 4 chars shown: 'sk_l...'");
    println!();

    // Demonstrate exposure when needed
    println!("2. Accessing the actual secret (when needed):");
    println!("   Actual value: {}", api_key.expose_secret());
    println!("   ⚠️  Use expose_secret() only when absolutely necessary");
    println!();

    // Multiple secrets
    let secrets = vec![
        "key1_secret_value".to_string(),
        "key2_different_value".to_string(),
        "key3_another_secret".to_string(),
    ];

    println!("3. Multiple secrets:");
    for (i, secret) in secrets.iter().enumerate() {
        let secret_obj = SecretString::from(secret.as_str());
        println!("   Secret {}: {:?}", i + 1, secret_obj);
    }
    println!("   ✓ All secrets redacted in Debug output");
    println!();

    // Simulated log scenario
    println!("4. Simulated log message:");
    let config_key = SecretString::new("api_key_for_production_server_xyz123".to_string());
    let log_msg = format!("Loading config with key: {:?}", config_key);
    println!("   {}", log_msg);
    println!("   ✓ Secrets safe in log files!");
    println!();

    println!("=== Benefits ===");
    println!("✓ Prevents accidental credential exposure in logs");
    println!("✓ Still shows enough info to identify different keys (first 4 chars)");
    println!("✓ Actual values accessible when needed via expose_secret()");
    println!("✓ Zero-cost abstraction - no runtime overhead");
}
