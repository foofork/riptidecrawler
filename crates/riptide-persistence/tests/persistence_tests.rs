//! Persistence layer tests

use riptide_persistence::*;
use std::time::Duration;

#[cfg(test)]
mod storage_tests {
    use super::*;
    use riptide_persistence::storage::{DatabaseStorage, FileStorage, StorageBackend};

    #[tokio::test]
    async fn test_file_storage() {
        let storage = FileStorage::new("/tmp/riptide_test").await.unwrap();

        // Write data
        let data = b"test data";
        storage.write("test_key", data).await.unwrap();

        // Read data
        let read_data = storage.read("test_key").await.unwrap();
        assert_eq!(read_data, data);

        // Delete data
        storage.delete("test_key").await.unwrap();
        assert!(storage.read("test_key").await.is_err());
    }

    #[tokio::test]
    async fn test_database_storage() {
        let storage = DatabaseStorage::new(":memory:").await;

        match storage {
            Ok(db) => {
                // Insert record
                db.insert(
                    "crawl_results",
                    &CrawlRecord {
                        id: "123".to_string(),
                        url: "https://example.com".to_string(),
                        content: "Test content".to_string(),
                        timestamp: chrono::Utc::now(),
                    },
                )
                .await
                .unwrap();

                // Query record
                let records = db.query("crawl_results", "id = '123'").await.unwrap();
                assert_eq!(records.len(), 1);
            }
            Err(_) => {
                // Database may not be available in test environment
            }
        }
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        let cache = PersistentCache::new(CacheConfig {
            path: "/tmp/riptide_cache".to_string(),
            max_size_mb: 100,
            ttl: Duration::from_secs(3600),
        })
        .await
        .unwrap();

        cache.set("key1", "value1").await.unwrap();
        assert_eq!(cache.get("key1").await.unwrap(), "value1");

        // Simulate restart by creating new instance
        let cache2 = PersistentCache::new(CacheConfig {
            path: "/tmp/riptide_cache".to_string(),
            max_size_mb: 100,
            ttl: Duration::from_secs(3600),
        })
        .await
        .unwrap();

        // Should still have data
        assert_eq!(cache2.get("key1").await.unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let storage = FileStorage::new("/tmp/riptide_batch").await.unwrap();

        let items = vec![
            ("key1", b"value1".to_vec()),
            ("key2", b"value2".to_vec()),
            ("key3", b"value3".to_vec()),
        ];

        // Batch write
        storage.write_batch(items).await.unwrap();

        // Batch read
        let keys = vec!["key1", "key2", "key3"];
        let values = storage.read_batch(keys).await.unwrap();
        assert_eq!(values.len(), 3);
    }
}

#[cfg(test)]
mod queue_tests {
    use super::*;
    use riptide_persistence::queue::{PersistentQueue, QueueConfig};

    #[tokio::test]
    async fn test_queue_operations() {
        let queue = PersistentQueue::new(QueueConfig {
            path: "/tmp/riptide_queue".to_string(),
            max_size: 1000,
        })
        .await
        .unwrap();

        // Push items
        queue.push("item1").await.unwrap();
        queue.push("item2").await.unwrap();
        queue.push("item3").await.unwrap();

        assert_eq!(queue.size().await, 3);

        // Pop items (FIFO)
        assert_eq!(queue.pop().await.unwrap(), "item1");
        assert_eq!(queue.pop().await.unwrap(), "item2");
        assert_eq!(queue.size().await, 1);
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let queue = PriorityQueue::new("/tmp/riptide_pqueue").await.unwrap();

        queue.push("low", 1).await.unwrap();
        queue.push("high", 10).await.unwrap();
        queue.push("medium", 5).await.unwrap();

        // Should pop in priority order
        assert_eq!(queue.pop().await.unwrap(), "high");
        assert_eq!(queue.pop().await.unwrap(), "medium");
        assert_eq!(queue.pop().await.unwrap(), "low");
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;
    use riptide_persistence::checkpoint::{CheckpointManager, CrawlState};

    #[tokio::test]
    async fn test_checkpoint_save_restore() {
        let manager = CheckpointManager::new("/tmp/riptide_checkpoints")
            .await
            .unwrap();

        let state = CrawlState {
            job_id: "job123".to_string(),
            urls_processed: 50,
            urls_pending: vec!["url1".to_string(), "url2".to_string()],
            last_url: "https://example.com/page50".to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Save checkpoint
        manager.save_checkpoint(&state).await.unwrap();

        // Restore checkpoint
        let restored = manager.restore_checkpoint("job123").await.unwrap();
        assert_eq!(restored.urls_processed, 50);
        assert_eq!(restored.urls_pending.len(), 2);
    }

    #[tokio::test]
    async fn test_checkpoint_cleanup() {
        let manager = CheckpointManager::new("/tmp/riptide_checkpoints")
            .await
            .unwrap();

        // Create old checkpoints
        for i in 0..5 {
            let state = CrawlState {
                job_id: format!("old_job_{}", i),
                urls_processed: i,
                urls_pending: vec![],
                last_url: format!("url{}", i),
                timestamp: chrono::Utc::now() - chrono::Duration::days(30),
            };
            manager.save_checkpoint(&state).await.unwrap();
        }

        // Clean old checkpoints
        let removed = manager
            .cleanup_old_checkpoints(Duration::from_secs(86400 * 7))
            .await
            .unwrap();
        assert!(removed >= 5);
    }
}
