"""
Worker/Job Management API Example

Demonstrates how to use the RipTide Workers API for job submission,
monitoring, and management.
"""

import asyncio
from riptide_sdk import (
    RipTideClient,
    JobConfig,
    JobType,
    JobPriority,
    JobStatus,
    ScheduledJobConfig,
    CrawlOptions,
    CacheMode,
)


async def basic_job_submission():
    """Example 1: Submit a simple batch crawl job"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create job configuration
        config = JobConfig(
            job_type=JobType.batch_crawl(
                urls=["https://example.com", "https://example.org"],
                options=CrawlOptions(
                    cache_mode=CacheMode.READ_WRITE,
                    concurrency=5
                )
            ),
            priority=JobPriority.HIGH,
        )

        # Submit job
        job_id = await client.workers.submit_job(config)
        print(f"Job submitted: {job_id}")

        # Get job status
        job = await client.workers.get_job_status(job_id)
        print(f"Job status: {job.status}")
        print(f"Created at: {job.created_at}")


async def wait_for_job_completion():
    """Example 2: Submit job and wait for completion"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Submit job
        config = JobConfig(
            job_type=JobType.single_crawl("https://example.com"),
            priority=JobPriority.NORMAL,
        )

        job_id = await client.workers.submit_job(config)
        print(f"Job submitted: {job_id}")

        # Wait for completion (polls every 2 seconds, timeout after 5 minutes)
        try:
            result = await client.workers.wait_for_job(
                job_id,
                poll_interval=2.0,
                timeout=300.0
            )

            if result.success:
                print(f"Job completed successfully in {result.processing_time_ms}ms")
                print(f"Result data: {result.data}")
            else:
                print(f"Job failed: {result.error}")

        except Exception as e:
            print(f"Error waiting for job: {e}")


async def list_and_filter_jobs():
    """Example 3: List jobs with filtering"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # List all pending jobs
        pending = await client.workers.list_jobs(
            status="pending",
            limit=20
        )
        print(f"\nPending jobs ({pending.total}):")
        for job in pending.jobs:
            print(f"  - {job.job_id}: {job.job_type} (priority: {job.priority})")

        # List completed batch crawl jobs
        completed = await client.workers.list_jobs(
            status="completed",
            job_type="batch_crawl",
            limit=10
        )
        print(f"\nCompleted batch crawls ({completed.total}):")
        for job in completed.jobs:
            duration = "N/A"
            if job.started_at and job.completed_at:
                # Calculate duration (simplified)
                duration = f"{job.completed_at} - {job.started_at}"
            print(f"  - {job.job_id}: {duration}")


async def queue_statistics():
    """Example 4: Get queue and worker statistics"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Get queue stats
        queue_stats = await client.workers.get_queue_stats()
        print("\nQueue Statistics:")
        print(f"  Pending: {queue_stats.pending}")
        print(f"  Processing: {queue_stats.processing}")
        print(f"  Completed: {queue_stats.completed}")
        print(f"  Failed: {queue_stats.failed}")
        print(f"  Retry: {queue_stats.retry}")
        print(f"  Delayed: {queue_stats.delayed}")
        print(f"  Total: {queue_stats.total}")

        # Get worker stats
        worker_stats = await client.workers.get_worker_stats()
        print("\nWorker Statistics:")
        print(f"  Total workers: {worker_stats.total_workers}")
        print(f"  Healthy workers: {worker_stats.healthy_workers}")
        print(f"  Jobs processed: {worker_stats.total_jobs_processed}")
        print(f"  Jobs failed: {worker_stats.total_jobs_failed}")
        print(f"  Is running: {worker_stats.is_running}")


async def scheduled_jobs():
    """Example 5: Create scheduled jobs with cron expressions"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Schedule a daily crawl at 2 AM
        config = ScheduledJobConfig(
            name="daily_news_crawl",
            cron_expression="0 2 * * *",  # Daily at 2 AM
            job_template=JobType.batch_crawl([
                "https://news.example.com",
                "https://blog.example.com",
            ]),
            priority=JobPriority.NORMAL,
            enabled=True,
        )

        scheduled = await client.workers.submit_scheduled_job(config)
        print(f"\nScheduled job created: {scheduled.id}")
        print(f"  Name: {scheduled.name}")
        print(f"  Cron: {scheduled.cron_expression}")
        print(f"  Next execution: {scheduled.next_execution_at}")
        print(f"  Execution count: {scheduled.execution_count}")


async def custom_jobs():
    """Example 6: Submit custom job types"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Maintenance job
        maintenance_config = JobConfig(
            job_type=JobType.maintenance(
                task_type="cache_cleanup",
                parameters={
                    "max_age_days": 30,
                    "dry_run": False,
                }
            ),
            priority=JobPriority.LOW,
        )

        job_id = await client.workers.submit_job(maintenance_config)
        print(f"Maintenance job submitted: {job_id}")

        # Custom job
        custom_config = JobConfig(
            job_type=JobType.custom(
                job_name="data_export",
                payload={
                    "format": "json",
                    "date_range": "2024-01-01/2024-12-31",
                    "include_metadata": True,
                }
            ),
            priority=JobPriority.NORMAL,
        )

        job_id = await client.workers.submit_job(custom_config)
        print(f"Custom job submitted: {job_id}")


async def advanced_job_monitoring():
    """Example 7: Advanced job monitoring with polling"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Submit multiple jobs
        job_ids = []
        for i in range(5):
            config = JobConfig(
                job_type=JobType.single_crawl(f"https://example.com/page{i}"),
                priority=JobPriority.NORMAL,
            )
            job_id = await client.workers.submit_job(config)
            job_ids.append(job_id)

        print(f"Submitted {len(job_ids)} jobs")

        # Monitor until all complete
        completed = 0
        while completed < len(job_ids):
            await asyncio.sleep(2)  # Poll every 2 seconds

            # Check each job
            for job_id in job_ids:
                job = await client.workers.get_job_status(job_id)

                if job.status == JobStatus.COMPLETED:
                    completed += 1
                    print(f"  ✓ {job_id}: Completed in {job.processing_time_ms}ms")
                elif job.status == JobStatus.FAILED:
                    completed += 1
                    print(f"  ✗ {job_id}: Failed - {job.last_error}")
                elif job.status == JobStatus.PROCESSING:
                    print(f"  ⏳ {job_id}: Processing...")


async def error_handling():
    """Example 8: Proper error handling"""
    from riptide_sdk import APIError, ValidationError, TimeoutError

    async with RipTideClient(base_url="http://localhost:8080") as client:
        try:
            # Attempt to get non-existent job
            job = await client.workers.get_job_status("invalid-job-id")
        except APIError as e:
            if e.status_code == 404:
                print(f"Job not found: {e.message}")
            else:
                print(f"API error: {e.message}")
        except ValidationError as e:
            print(f"Validation error: {e}")
        except TimeoutError as e:
            print(f"Timeout: {e}")


async def main():
    """Run all examples"""
    print("=== RipTide Workers API Examples ===\n")

    # Run examples
    examples = [
        ("Basic Job Submission", basic_job_submission),
        ("Wait for Job Completion", wait_for_job_completion),
        ("List and Filter Jobs", list_and_filter_jobs),
        ("Queue Statistics", queue_statistics),
        ("Scheduled Jobs", scheduled_jobs),
        ("Custom Jobs", custom_jobs),
        ("Advanced Monitoring", advanced_job_monitoring),
        ("Error Handling", error_handling),
    ]

    for name, example_func in examples:
        print(f"\n{'='*60}")
        print(f"Example: {name}")
        print(f"{'='*60}")
        try:
            await example_func()
        except Exception as e:
            print(f"Error in {name}: {e}")
            import traceback
            traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())
