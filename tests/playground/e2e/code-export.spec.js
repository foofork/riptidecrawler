import { test, expect } from '@playwright/test'

test.describe('Code Export E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.click('text=API Playground')
    await page.click('text=Start Crawl')
  })

  test('should generate JavaScript code', async ({ page }) => {
    // Click on Code tab
    await page.click('text=Code')

    // Select JavaScript
    await page.click('button:has-text("Javascript")')

    // Verify code is generated
    const codeEditor = page.locator('.cm-content')
    await expect(codeEditor).toContainText('fetch')
    await expect(codeEditor).toContainText('axios')
    await expect(codeEditor).toContainText('/crawl')
  })

  test('should generate Python code', async ({ page }) => {
    await page.click('text=Code')

    // Select Python
    await page.click('button:has-text("Python")')

    // Verify Python code
    const codeEditor = page.locator('.cm-content')
    await expect(codeEditor).toContainText('import requests')
    await expect(codeEditor).toContainText('from riptide import RipTide')
  })

  test('should generate cURL commands', async ({ page }) => {
    await page.click('text=Code')

    // Select cURL
    await page.click('button:has-text("Curl")')

    // Verify cURL code
    const codeEditor = page.locator('.cm-content')
    await expect(codeEditor).toContainText('curl -X POST')
    await expect(codeEditor).toContainText('-H')
    await expect(codeEditor).toContainText('jq')
  })

  test('should generate Rust code', async ({ page }) => {
    await page.click('text=Code')

    // Select Rust
    await page.click('button:has-text("Rust")')

    // Verify Rust code
    const codeEditor = page.locator('.cm-content')
    await expect(codeEditor).toContainText('use reqwest')
    await expect(codeEditor).toContainText('#[tokio::main]')
    await expect(codeEditor).toContainText('async fn main')
  })

  test('should update code when endpoint changes', async ({ page }) => {
    await page.click('text=Code')

    // Get initial code
    const codeEditor = page.locator('.cm-content')
    const initialText = await codeEditor.textContent()

    // Change endpoint
    await page.click('text=Response') // Switch tab first
    await page.click('text=Get Job Status')

    // Go back to code tab
    await page.click('text=Code')

    // Verify code changed
    const newText = await codeEditor.textContent()
    expect(newText).not.toBe(initialText)
    await expect(codeEditor).toContainText('/job/:jobId')
  })
})
