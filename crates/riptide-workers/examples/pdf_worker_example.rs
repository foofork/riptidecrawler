use riptide_workers::prelude::*;
use riptide_workers::{PdfProcessor, PdfExtractionOptions, SchedulerConfig};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting PDF worker example");

    // Create worker service configuration
    let config = WorkerServiceConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        worker_config: WorkerConfig {
            worker_count: 2,
            poll_interval_secs: 2,
            job_timeout_secs: 300, // 5 minutes for PDF processing
            max_concurrent_jobs: 1, // One PDF job per worker to manage memory
            ..Default::default()
        },
        queue_config: QueueConfig::default(),
        scheduler_config: SchedulerConfig::default(),
        max_batch_size: 10,
        max_concurrency: 5,
        wasm_path: "./wasm/riptide-extractor.wasm".to_string(),
        enable_scheduler: false,
    };

    // Create worker service
    let mut service = WorkerService::new(config).await?;

    // PDF processor would be registered during service initialization
    // Note: add_processor method has been removed in the current API

    info!("PDF worker service configured, starting processing");

    // Example PDF job submission
    let sample_pdf_data = create_sample_pdf_data();

    let pdf_job = Job::new(JobType::PdfExtraction {
        pdf_data: sample_pdf_data,
        url: Some("example.pdf".to_string()),
        options: Some(PdfExtractionOptions {
            extract_text: true,
            extract_images: false,
            extract_metadata: true,
            max_size_bytes: 50 * 1024 * 1024, // 50MB
            enable_progress: true,
            custom_settings: std::collections::HashMap::new(),
        }),
    });
    // Note: with_priority method has been removed; priority is set via JobType

    // Submit the job
    let job_id = service.submit_job(pdf_job).await?;
    info!("Submitted PDF extraction job: {}", job_id);

    // Run the service for a short time for demonstration
    tokio::select! {
        result = service.start() => {
            match result {
                Ok(()) => info!("Worker service completed"),
                Err(e) => eprintln!("Worker service error: {}", e),
            }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
            info!("Example timeout reached, stopping service");
            service.stop().await?;
        }
    }

    info!("PDF worker example completed");
    Ok(())
}

/// Create sample PDF data for demonstration
/// In a real application, this would be actual PDF bytes
fn create_sample_pdf_data() -> Vec<u8> {
    // This is a minimal PDF structure for demonstration
    // In practice, you'd load real PDF files
    let pdf_header = b"%PDF-1.4\n";
    let pdf_body = b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";
    let pdf_trailer = b"xref\n0 2\n0000000000 65535 f \n0000000009 00000 n \n";
    let pdf_end = b"trailer\n<< /Size 2 /Root 1 0 R >>\nstartxref\n47\n%%EOF\n";

    let mut pdf_data = Vec::new();
    pdf_data.extend_from_slice(pdf_header);
    pdf_data.extend_from_slice(pdf_body);
    pdf_data.extend_from_slice(pdf_trailer);
    pdf_data.extend_from_slice(pdf_end);

    pdf_data
}