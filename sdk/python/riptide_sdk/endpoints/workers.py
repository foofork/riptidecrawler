"""
Worker/Job management API endpoint implementation

Provides comprehensive job submission, monitoring, and management capabilities
for the RipTide worker queue system.
"""

from typing import Optional, List
import httpx

from ..models import (
    Job,
    JobConfig,
    JobResult,
    QueueStats,
    WorkerStats,
    ScheduledJob,
    ScheduledJobConfig,
    JobListResponse,
)
from ..exceptions import APIError, ValidationError


class WorkersAPI:
    """API for worker and job management operations"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize WorkersAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def submit_job(self, config: JobConfig) -> str:
        """
        Submit a job to the worker queue

        Args:
            config: Job configuration including type, priority, and options

        Returns:
            Job ID (UUID string)

        Raises:
            ValidationError: If job configuration is invalid
            APIError: If the API returns an error

        Example:
            >>> from riptide_sdk.models import JobConfig, JobType, JobPriority
            >>>
            >>> # Submit a batch crawl job
            >>> config = JobConfig(
            ...     job_type=JobType.batch_crawl(
            ...         ["https://example.com", "https://example.org"]
            ...     ),
            ...     priority=JobPriority.HIGH
            ... )
            >>> job_id = await client.workers.submit_job(config)
            >>> print(f"Job submitted: {job_id}")
        """
        # Make request
        response = await self.client.post(
            f"{self.base_url}/api/v1/workers/jobs",
            json=config.to_dict(),
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Job submission failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        data = response.json()
        return data['job_id']

    async def list_jobs(
        self,
        status: Optional[str] = None,
        job_type: Optional[str] = None,
        limit: int = 50,
        offset: int = 0,
        search: Optional[str] = None,
    ) -> JobListResponse:
        """
        List jobs with filtering and pagination

        Args:
            status: Filter by job status (pending, processing, completed, failed, etc.)
            job_type: Filter by job type (batch_crawl, single_crawl, etc.)
            limit: Maximum number of jobs to return (default: 50, max: 500)
            offset: Number of jobs to skip (default: 0)
            search: Search term for filtering jobs

        Returns:
            JobListResponse with jobs and pagination info

        Raises:
            APIError: If the API returns an error

        Example:
            >>> # List all pending jobs
            >>> result = await client.workers.list_jobs(status="pending", limit=20)
            >>> print(f"Found {result.total} pending jobs")
            >>> for job in result.jobs:
            ...     print(f"  - {job.job_id}: {job.job_type}")
            >>>
            >>> # List completed crawl jobs
            >>> result = await client.workers.list_jobs(
            ...     status="completed",
            ...     job_type="batch_crawl",
            ...     limit=100
            ... )
        """
        # Build query parameters
        params = {
            "limit": min(limit, 500),
            "offset": offset,
        }
        if status:
            params["status"] = status
        if job_type:
            params["job_type"] = job_type
        if search:
            params["search"] = search

        # Make request
        response = await self.client.get(
            f"{self.base_url}/api/v1/workers/jobs",
            params=params,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Failed to list jobs"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return JobListResponse.from_dict(response.json())

    async def get_job_status(self, job_id: str) -> Job:
        """
        Get status of a specific job

        Args:
            job_id: Job ID (UUID string)

        Returns:
            Job object with current status and metadata

        Raises:
            APIError: If job not found or API error occurs

        Example:
            >>> job = await client.workers.get_job_status(job_id)
            >>> print(f"Status: {job.status}")
            >>> print(f"Created: {job.created_at}")
            >>> if job.started_at:
            ...     print(f"Started: {job.started_at}")
            >>> if job.processing_time_ms:
            ...     print(f"Processing time: {job.processing_time_ms}ms")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/workers/jobs/{job_id}",
        )

        if response.status_code == 404:
            raise APIError(
                message=f"Job not found: {job_id}",
                status_code=404,
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Failed to get job status"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return Job.from_dict(response.json())

    async def get_job_result(self, job_id: str) -> JobResult:
        """
        Get result of a completed job

        Args:
            job_id: Job ID (UUID string)

        Returns:
            JobResult with execution outcome and data

        Raises:
            APIError: If job not found, not completed, or API error occurs

        Example:
            >>> result = await client.workers.get_job_result(job_id)
            >>> if result.success:
            ...     print(f"Job completed successfully in {result.processing_time_ms}ms")
            ...     print(f"Result data: {result.data}")
            ... else:
            ...     print(f"Job failed: {result.error}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/workers/jobs/{job_id}/result",
        )

        if response.status_code == 404:
            raise APIError(
                message=f"Job result not found: {job_id}",
                status_code=404,
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Failed to get job result"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return JobResult.from_dict(response.json())

    async def get_queue_stats(self) -> QueueStats:
        """
        Get queue statistics

        Returns:
            QueueStats with counts for each queue state

        Raises:
            APIError: If the API returns an error

        Example:
            >>> stats = await client.workers.get_queue_stats()
            >>> print(f"Pending: {stats.pending}")
            >>> print(f"Processing: {stats.processing}")
            >>> print(f"Completed: {stats.completed}")
            >>> print(f"Failed: {stats.failed}")
            >>> print(f"Total: {stats.total}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/workers/queue/stats",
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Failed to get queue stats"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return QueueStats.from_dict(response.json())

    async def get_worker_stats(self) -> WorkerStats:
        """
        Get worker pool statistics

        Returns:
            WorkerStats with worker pool information

        Raises:
            APIError: If the API returns an error

        Example:
            >>> stats = await client.workers.get_worker_stats()
            >>> print(f"Total workers: {stats.total_workers}")
            >>> print(f"Healthy workers: {stats.healthy_workers}")
            >>> print(f"Jobs processed: {stats.total_jobs_processed}")
            >>> print(f"Jobs failed: {stats.total_jobs_failed}")
            >>> print(f"Is running: {stats.is_running}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/workers/stats",
        )

        if response.status_code == 503:
            raise APIError(
                message="Worker pool not yet started",
                status_code=503,
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Failed to get worker stats"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return WorkerStats.from_dict(response.json())

    async def create_scheduled_job(self, config: ScheduledJobConfig) -> ScheduledJob:
        """
        Create a scheduled job with cron expression

        Args:
            config: Scheduled job configuration

        Returns:
            ScheduledJob object with scheduling details

        Raises:
            ValidationError: If cron expression is invalid
            APIError: If the API returns an error

        Example:
            >>> from riptide_sdk.models import (
            ...     ScheduledJobConfig,
            ...     JobType,
            ...     JobPriority,
            ... )
            >>>
            >>> # Schedule a daily batch crawl at 2 AM
            >>> config = ScheduledJobConfig(
            ...     name="daily_crawl",
            ...     cron_expression="0 2 * * *",
            ...     job_template=JobType.batch_crawl(["https://example.com"]),
            ...     priority=JobPriority.NORMAL,
            ...     enabled=True,
            ... )
            >>> scheduled = await client.workers.create_scheduled_job(config)
            >>> print(f"Scheduled job created: {scheduled.id}")
            >>> print(f"Next execution: {scheduled.next_execution_at}")
        """
        response = await self.client.post(
            f"{self.base_url}/api/v1/workers/scheduled",
            json=config.to_dict(),
        )

        if response.status_code == 400:
            error_data = response.json() if response.text else {}
            raise ValidationError(
                error_data.get("message", "Invalid cron expression or configuration")
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("message", "Failed to create scheduled job"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return ScheduledJob.from_dict(response.json())

    async def wait_for_job(
        self,
        job_id: str,
        poll_interval: float = 1.0,
        timeout: Optional[float] = None,
    ) -> JobResult:
        """
        Wait for a job to complete and return its result

        This is a convenience method that polls the job status until completion
        or timeout. For production use, consider using webhooks or message queues.

        Args:
            job_id: Job ID to wait for
            poll_interval: Seconds between status checks (default: 1.0)
            timeout: Maximum seconds to wait (default: None = wait forever)

        Returns:
            JobResult when job completes

        Raises:
            TimeoutError: If timeout is reached
            APIError: If job fails or API error occurs

        Example:
            >>> # Submit job and wait for completion
            >>> job_id = await client.workers.submit_job(config)
            >>> result = await client.workers.wait_for_job(
            ...     job_id,
            ...     poll_interval=2.0,
            ...     timeout=300.0  # 5 minutes
            ... )
            >>> print(f"Job completed: {result.success}")
        """
        import asyncio
        from ..exceptions import TimeoutError as RipTideTimeoutError
        from ..models import JobStatus

        start_time = asyncio.get_event_loop().time()

        while True:
            # Check timeout
            if timeout is not None:
                elapsed = asyncio.get_event_loop().time() - start_time
                if elapsed >= timeout:
                    raise RipTideTimeoutError(
                        f"Job {job_id} did not complete within {timeout} seconds"
                    )

            # Get job status
            job = await self.get_job_status(job_id)

            # Check if completed
            if job.status == JobStatus.COMPLETED:
                return await self.get_job_result(job_id)
            elif job.status in (JobStatus.FAILED, JobStatus.DEAD_LETTER):
                raise APIError(
                    message=f"Job failed: {job.last_error}",
                    status_code=500,
                )

            # Wait before next poll
            await asyncio.sleep(poll_interval)
