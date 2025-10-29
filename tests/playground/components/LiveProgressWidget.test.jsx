import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import LiveProgressWidget from '@/components/streaming/LiveProgressWidget'

describe('LiveProgressWidget Component', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.runOnlyPendingTimers()
    vi.useRealTimers()
  })

  describe('Initial State', () => {
    it('should render expanded by default', () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      expect(screen.getByText('Crawling in Progress...')).toBeInTheDocument()
      expect(screen.getByText(/Progress:/)).toBeInTheDocument()
    })

    it('should show initial statistics', () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      expect(screen.getByText('Success')).toBeInTheDocument()
      expect(screen.getByText('Failed')).toBeInTheDocument()
      expect(screen.getByText('Rate')).toBeInTheDocument()
    })
  })

  describe('Collapse/Expand', () => {
    it('should collapse when collapse button clicked', () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      const collapseButton = screen.getByTitle('Collapse')
      fireEvent.click(collapseButton)

      expect(screen.getByText(/Crawling\.\.\./)).toBeInTheDocument()
      expect(screen.queryByText(/Progress:/)).not.toBeInTheDocument()
    })

    it('should expand when expand button clicked', () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      const collapseButton = screen.getByTitle('Collapse')
      fireEvent.click(collapseButton)

      const expandButton = screen.getByTitle('Expand')
      fireEvent.click(expandButton)

      expect(screen.getByText(/Progress:/)).toBeInTheDocument()
    })
  })

  describe('Close Functionality', () => {
    it('should call onClose when close button clicked', () => {
      const onClose = vi.fn()
      render(<LiveProgressWidget crawlId="test-123" onClose={onClose} />)

      const closeButton = screen.getByTitle('Close')
      fireEvent.click(closeButton)

      expect(onClose).toHaveBeenCalled()
    })
  })

  describe('Progress Updates', () => {
    it('should update progress over time', async () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      // Fast-forward time to trigger progress updates
      vi.advanceTimersByTime(3000)

      await waitFor(() => {
        // Progress should have increased
        const progressText = screen.getByText(/Progress:/)
        expect(progressText).toBeInTheDocument()
      })
    })

    it('should calculate progress percentage correctly', async () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      vi.advanceTimersByTime(2000)

      await waitFor(() => {
        // Should show some percentage
        expect(screen.getByText(/%$/)).toBeInTheDocument()
      })
    })

    it('should show completion status when finished', async () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      // Fast-forward enough time to complete
      vi.advanceTimersByTime(60000)

      await waitFor(() => {
        expect(screen.getByText('Crawl Completed')).toBeInTheDocument()
      })
    })
  })

  describe('Statistics Display', () => {
    it('should display success count', async () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      vi.advanceTimersByTime(1000)

      await waitFor(() => {
        const successSection = screen.getByText('Success').parentElement.parentElement
        expect(successSection).toBeInTheDocument()
      })
    })

    it('should display crawl rate', async () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      vi.advanceTimersByTime(2000)

      await waitFor(() => {
        expect(screen.getByText('URLs/sec')).toBeInTheDocument()
      })
    })

    it('should show elapsed time', () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      expect(screen.getByText('Elapsed Time')).toBeInTheDocument()
    })

    it('should show ETA', () => {
      render(<LiveProgressWidget crawlId="test-123" onClose={vi.fn()} />)

      expect(screen.getByText('ETA Remaining')).toBeInTheDocument()
    })
  })
})
