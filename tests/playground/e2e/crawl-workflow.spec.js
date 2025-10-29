import { test, expect } from '@playwright/test'

test.describe('Crawl Workflow E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should complete full crawl workflow', async ({ page }) => {
    // Navigate to playground
    await page.click('text=API Playground')

    // Select crawl endpoint
    await page.click('text=Start Crawl')

    // Wait for request builder to load
    await expect(page.locator('text=Request Body')).toBeVisible()

    // Modify request body
    const editor = page.locator('.cm-content')
    await editor.click()
    await page.keyboard.press('Control+A')
    await page.keyboard.type(JSON.stringify({
      url: 'https://example.com',
      options: {
        depth: 1,
        followLinks: true
      }
    }))

    // Execute request
    await page.click('button:has-text("Execute")')

    // Wait for response
    await expect(page.locator('text=200 OK')).toBeVisible({ timeout: 10000 })

    // Verify response contains jobId
    const response = page.locator('[role="textbox"]').first()
    await expect(response).toContainText('jobId')
  })

  test('should handle crawl with path parameters', async ({ page }) => {
    await page.click('text=API Playground')

    // Select job status endpoint
    await page.click('text=Get Job Status')

    // Fill path parameter
    await page.fill('input[placeholder*="jobId"]', 'test-job-123')

    // Execute request
    await page.click('button:has-text("Execute")')

    // Verify request executed
    await expect(page.locator('[class*="animate-spin"]')).toBeVisible()
  })

  test('should display validation errors for invalid JSON', async ({ page }) => {
    await page.click('text=API Playground')
    await page.click('text=Start Crawl')

    // Enter invalid JSON
    const editor = page.locator('.cm-content')
    await editor.click()
    await page.keyboard.press('Control+A')
    await page.keyboard.type('invalid json {')

    // Execute request
    await page.click('button:has-text("Execute")')

    // Should show error
    await expect(page.locator('text=Invalid JSON')).toBeVisible({ timeout: 5000 })
  })

  test('should show request timing information', async ({ page }) => {
    await page.click('text=API Playground')
    await page.click('text=Start Crawl')

    await page.click('button:has-text("Execute")')

    // Wait for response and check for duration
    await expect(page.locator('text=/\\d+ms/')).toBeVisible({ timeout: 10000 })
  })
})
