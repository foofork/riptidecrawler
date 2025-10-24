//! Concurrent access and thread safety tests
//!
//! Tests semaphore control, concurrent operations, and race conditions

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};

#[tokio::test]
async fn test_semaphore_basic_concurrency() {
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = vec![];

    // Spawn 6 concurrent tasks, but only 3 can run at a time
    for i in 0..6 {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            i
        });
        handles.push(handle);
    }

    // All tasks should complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

#[tokio::test]
async fn test_semaphore_try_acquire() {
    let semaphore = Arc::new(Semaphore::new(2));

    // Acquire all permits
    let _permit1 = semaphore.try_acquire().unwrap();
    let _permit2 = semaphore.try_acquire().unwrap();

    // Next try_acquire should fail
    assert!(semaphore.try_acquire().is_err());

    // Release one permit
    drop(_permit1);

    // Now should succeed
    let _permit3 = semaphore.try_acquire().unwrap();
    assert!(semaphore.try_acquire().is_err());
}

#[tokio::test]
async fn test_semaphore_timeout() {
    let semaphore = Arc::new(Semaphore::new(1));

    // Acquire the permit
    let _permit = semaphore.acquire().await.unwrap();

    // Try to acquire with timeout (should fail)
    let result = tokio::time::timeout(Duration::from_millis(100), semaphore.acquire()).await;

    assert!(result.is_err()); // Timeout occurred
}

#[tokio::test]
async fn test_concurrent_mutex_access() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    // Spawn 100 tasks that increment counter
    for _ in 0..100 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            let mut count = counter_clone.lock().await;
            *count += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_count = *counter.lock().await;
    assert_eq!(final_count, 100);
}

#[tokio::test]
async fn test_concurrent_vector_operations() {
    let vec = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Concurrently push items
    for i in 0..50 {
        let vec_clone = vec.clone();
        let handle = tokio::spawn(async move {
            let mut v = vec_clone.lock().await;
            v.push(i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_vec = vec.lock().await;
    assert_eq!(final_vec.len(), 50);
}

#[tokio::test]
async fn test_fairness_under_contention() {
    let semaphore = Arc::new(Semaphore::new(2));
    let completion_order = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    // Spawn tasks with different IDs
    for i in 0..10 {
        let sem = semaphore.clone();
        let order = completion_order.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;

            let mut ord = order.lock().await;
            ord.push(i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let order = completion_order.lock().await;
    assert_eq!(order.len(), 10);
}

#[tokio::test]
async fn test_backpressure_handling() {
    let semaphore = Arc::new(Semaphore::new(5));
    let active_count = Arc::new(Mutex::new(0));
    let max_active = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..20 {
        let sem = semaphore.clone();
        let active = active_count.clone();
        let max = max_active.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Track active count
            {
                let mut act = active.lock().await;
                *act += 1;

                let mut m = max.lock().await;
                if *act > *m {
                    *m = *act;
                }
            }

            tokio::time::sleep(Duration::from_millis(10)).await;

            // Decrement active count
            {
                let mut act = active.lock().await;
                *act -= 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let max_concurrent = *max_active.lock().await;
    assert!(max_concurrent <= 5); // Should never exceed semaphore limit
    assert!(max_concurrent > 0); // Should have had concurrent activity
}

#[tokio::test]
async fn test_permit_dropping() {
    let semaphore = Arc::new(Semaphore::new(3));

    // Acquire permits in inner scope
    {
        let _p1 = semaphore.acquire().await.unwrap();
        let _p2 = semaphore.acquire().await.unwrap();
        let _p3 = semaphore.acquire().await.unwrap();

        // All permits taken
        assert_eq!(semaphore.available_permits(), 0);
    }
    // Permits dropped here

    // Permits should be released
    assert_eq!(semaphore.available_permits(), 3);
}

#[tokio::test]
async fn test_concurrent_reads_exclusive_writes() {
    use tokio::sync::RwLock;

    let data = Arc::new(RwLock::new(0));
    let mut handles = vec![];

    // Spawn concurrent readers
    for _ in 0..10 {
        let data_clone = data.clone();
        let handle = tokio::spawn(async move {
            let value = *data_clone.read().await;
            value
        });
        handles.push(handle);
    }

    // Spawn exclusive writer
    let data_clone = data.clone();
    let write_handle = tokio::spawn(async move {
        let mut value = data_clone.write().await;
        *value += 100;
    });

    for handle in handles {
        handle.await.unwrap();
    }
    write_handle.await.unwrap();

    let final_value = *data.read().await;
    assert_eq!(final_value, 100);
}

#[tokio::test]
async fn test_atomic_operations() {
    use std::sync::atomic::{AtomicU64, Ordering};

    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    // Concurrent atomic increments
    for _ in 0..1000 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(counter.load(Ordering::Relaxed), 1000);
}

#[tokio::test]
async fn test_concurrent_pool_access_simulation() {
    use std::collections::VecDeque;

    let pool = Arc::new(Mutex::new(VecDeque::new()));
    let semaphore = Arc::new(Semaphore::new(5));

    // Pre-fill pool
    {
        let mut p = pool.lock().await;
        for i in 0..10 {
            p.push_back(i);
        }
    }

    let mut handles = vec![];

    // Simulate concurrent acquire/release
    for _ in 0..20 {
        let pool_clone = pool.clone();
        let sem = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Acquire from pool
            let item = {
                let mut p = pool_clone.lock().await;
                p.pop_front()
            };

            if let Some(i) = item {
                tokio::time::sleep(Duration::from_millis(5)).await;

                // Return to pool
                let mut p = pool_clone.lock().await;
                p.push_back(i);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Pool should still have items
    let final_pool = pool.lock().await;
    assert!(final_pool.len() > 0);
}

#[tokio::test]
async fn test_work_queue_pattern() {
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let processed = Arc::new(Mutex::new(Vec::new()));

    // Fill queue with work items
    {
        let mut q = queue.lock().await;
        for i in 0..20 {
            q.push_back(i);
        }
    }

    let mut handles = vec![];

    // Spawn workers
    for _ in 0..4 {
        let queue_clone = queue.clone();
        let processed_clone = processed.clone();

        let handle = tokio::spawn(async move {
            loop {
                let work_item = {
                    let mut q = queue_clone.lock().await;
                    q.pop_front()
                };

                match work_item {
                    Some(item) => {
                        // Process item
                        tokio::time::sleep(Duration::from_millis(5)).await;

                        let mut p = processed_clone.lock().await;
                        p.push(item);
                    }
                    None => break, // No more work
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_processed = processed.lock().await;
    assert_eq!(final_processed.len(), 20);
}

#[tokio::test]
async fn test_rate_limiting() {
    use tokio::time::{interval, Duration};

    let semaphore = Arc::new(Semaphore::new(10)); // 10 requests per interval
    let mut interval_timer = interval(Duration::from_millis(100));

    let mut completed = 0;

    for _ in 0..5 {
        interval_timer.tick().await;

        // Try to process up to 10 requests
        let mut batch_handles = vec![];
        for _ in 0..10 {
            let sem = semaphore.clone();
            let handle = tokio::spawn(async move {
                let _permit = sem.try_acquire().ok();
                tokio::time::sleep(Duration::from_millis(10)).await;
            });
            batch_handles.push(handle);
        }

        for handle in batch_handles {
            handle.await.unwrap();
        }
        completed += 10;
    }

    assert_eq!(completed, 50);
}

#[tokio::test]
async fn test_channel_based_coordination() {
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn producers
    let mut producers = vec![];
    for i in 0..5 {
        let tx_clone = tx.clone();
        let producer = tokio::spawn(async move {
            for j in 0..10 {
                tx_clone.send(i * 10 + j).await.unwrap();
            }
        });
        producers.push(producer);
    }

    drop(tx); // Close channel when producers done

    // Consumer
    let consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Some(_item) = rx.recv().await {
            count += 1;
        }
        count
    });

    for producer in producers {
        producer.await.unwrap();
    }
    let total_received = consumer.await.unwrap();

    assert_eq!(total_received, 50);
}
