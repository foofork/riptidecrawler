-- Add retry tracking fields to event_outbox table
-- This migration adds support for exponential backoff retry logic

-- Add last_retry_at column for backoff delay calculation
ALTER TABLE event_outbox
ADD COLUMN IF NOT EXISTS last_retry_at TIMESTAMPTZ;

-- Add index for efficient polling with backoff
-- This index is used by the OutboxPublisher to find events ready for retry
CREATE INDEX IF NOT EXISTS idx_outbox_retry_backoff
ON event_outbox (last_retry_at)
WHERE published_at IS NULL AND retry_count < 10;

-- Add comment for documentation
COMMENT ON COLUMN event_outbox.last_retry_at IS 'Timestamp of last retry attempt, used for exponential backoff calculation';
COMMENT ON INDEX idx_outbox_retry_backoff IS 'Index for efficient outbox polling with exponential backoff support';
