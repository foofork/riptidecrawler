import { test, expect } from '@playwright/test'

test.describe('Streaming Workflow E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should connect to SSE stream and receive updates', async ({ page }) => {
    // Navigate to streaming page
    await page.click('text=Streaming')

    // Wait for streaming interface to load
    await expect(page.locator('h1:has-text("Live Streaming")')).toBeVisible()

    // Connect to stream
    await page.click('button:has-text("Connect")')

    // Verify connection status
    await expect(page.locator('text=Connected')).toBeVisible({ timeout: 5000 })

    // Verify stream updates
    await expect(page.locator('[data-testid="stream-data"]')).not.toBeEmpty({ timeout: 10000 })
  })

  test('should display live progress widget during crawl', async ({ page }) => {
    await page.click('text=Streaming')

    // Start crawl that triggers progress widget
    await page.click('button:has-text("Start Crawl")')

    // Verify progress widget appears
    await expect(page.locator('text=Crawling in Progress')).toBeVisible({ timeout: 3000 })

    // Verify progress updates
    await expect(page.locator('text=/Progress: \\d+\\/\\d+ URLs/')).toBeVisible()

    // Verify statistics are shown
    await expect(page.locator('text=Success')).toBeVisible()
    await expect(page.locator('text=Failed')).toBeVisible()
    await expect(page.locator('text=Rate')).toBeVisible()
  })

  test('should collapse and expand progress widget', async ({ page }) => {
    await page.click('text=Streaming')
    await page.click('button:has-text("Start Crawl")')

    await expect(page.locator('text=Crawling in Progress')).toBeVisible()

    // Collapse widget
    await page.click('[title="Collapse"]')

    // Verify compact view
    await expect(page.locator('text=/Crawling\\.\\.\\. \\d+%/')).toBeVisible()
    await expect(page.locator('text=/Progress: \\d+\\/\\d+/')).not.toBeVisible()

    // Expand widget
    await page.click('[title="Expand"]')

    // Verify full view restored
    await expect(page.locator('text=/Progress: \\d+\\/\\d+/')).toBeVisible()
  })
})
