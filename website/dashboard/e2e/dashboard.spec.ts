import { test, expect } from '@playwright/test'

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('loads and displays header', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Chub')
    await expect(page.locator('h1')).toContainText('Tracking Dashboard')
  })

  test('renders all four stat cards', async ({ page }) => {
    await expect(page.getByText('Active Session')).toBeVisible()
    await expect(page.getByText('Total Sessions')).toBeVisible()
    await expect(page.getByText('Estimated Cost')).toBeVisible()
    await expect(page.getByText('Total Tokens')).toBeVisible()
  })

  test('stat cards show data from API', async ({ page }) => {
    // Wait for data to load — session count should appear
    await expect(page.getByText('Total Sessions').locator('..').locator('..')).toBeVisible()
    // Cost card should show a dollar amount
    await expect(page.locator('text=/\\$\\d/').first()).toBeVisible()
  })

  test('renders breakdown chart section', async ({ page }) => {
    await expect(page.getByText('By Agent')).toBeVisible()
    await expect(page.getByText('By Model')).toBeVisible()
    await expect(page.getByText('Top Tools')).toBeVisible()
  })

  test('renders session history table', async ({ page }) => {
    await expect(page.getByText('Session History')).toBeVisible()
    // Wait for data to load, then check for table or empty state
    await page.waitForTimeout(2000)
    const hasTable = await page.locator('table').count()
    const hasEmpty = await page.getByText('No sessions found').count()
    expect(hasTable + hasEmpty).toBeGreaterThan(0)
  })

  test('session table shows session data when available', async ({ page }) => {
    // If sessions exist, the table should have rows with agent badges
    const tableRows = page.locator('table tbody tr')
    const rowCount = await tableRows.count()
    if (rowCount > 0) {
      // Each row should have a session ID (monospace), agent badge, and cost
      await expect(tableRows.first().locator('td').first()).toBeVisible()
    }
  })

  test('time range selector works', async ({ page }) => {
    // Open the select dropdown
    const trigger = page.locator('[data-slot="select-trigger"]')
    await expect(trigger).toBeVisible()
    await expect(trigger).toContainText('30')
    await trigger.click()

    // Select 7 days
    const option = page.getByText('Last 7 days')
    await expect(option).toBeVisible()
    await option.click()

    // Verify selector updated
    await expect(trigger).toContainText('7')
  })

  test('refresh button exists and is clickable', async ({ page }) => {
    const refreshBtn = page.locator('button[title="Refresh"]')
    await expect(refreshBtn).toBeVisible()
    await refreshBtn.click()
  })

  test('theme toggle cycles through system, light, dark', async ({ page }) => {
    // Default is system — icon should be Monitor
    const themeBtn = page.locator('button[title^="Theme:"]')
    await expect(themeBtn).toBeVisible()

    // Cycle to light
    await themeBtn.click()
    await expect(themeBtn).toHaveAttribute('title', 'Theme: light')
    await expect(page.locator('html')).not.toHaveClass(/dark/)

    // Cycle to dark
    await themeBtn.click()
    await expect(themeBtn).toHaveAttribute('title', 'Theme: dark')
    await expect(page.locator('html')).toHaveClass(/dark/)

    // Cycle back to system
    await themeBtn.click()
    await expect(themeBtn).toHaveAttribute('title', 'Theme: system')
  })

  test('theme persists across page reload', async ({ page }) => {
    const themeBtn = page.locator('button[title^="Theme:"]')
    // Set to dark
    await themeBtn.click() // system -> light
    await themeBtn.click() // light -> dark
    await expect(themeBtn).toHaveAttribute('title', 'Theme: dark')

    // Reload page
    await page.reload()
    const themeBtnAfter = page.locator('button[title^="Theme:"]')
    await expect(themeBtnAfter).toHaveAttribute('title', 'Theme: dark')
    await expect(page.locator('html')).toHaveClass(/dark/)
  })

  test('footer shows version and auto-refresh info', async ({ page }) => {
    await expect(page.getByText('Auto-refreshes every 10s')).toBeVisible()
    await expect(page.getByText('API at /api/*')).toBeVisible()
  })
})

test.describe('API endpoints', () => {
  test('/api/status returns valid JSON', async ({ request }) => {
    const res = await request.get('/api/status')
    expect(res.ok()).toBeTruthy()
    const data = await res.json()
    expect(data).toHaveProperty('agent_detected')
    expect(data).toHaveProperty('entire_sessions')
    expect(data).toHaveProperty('model_detected')
  })

  test('/api/sessions returns array', async ({ request }) => {
    const res = await request.get('/api/sessions?days=30')
    expect(res.ok()).toBeTruthy()
    const data = await res.json()
    expect(Array.isArray(data)).toBe(true)
    if (data.length > 0) {
      expect(data[0]).toHaveProperty('session_id')
      expect(data[0]).toHaveProperty('agent')
      expect(data[0]).toHaveProperty('tokens')
      expect(data[0]).toHaveProperty('tool_calls')
    }
  })

  test('/api/report returns correct shape', async ({ request }) => {
    const res = await request.get('/api/report?days=30')
    expect(res.ok()).toBeTruthy()
    const data = await res.json()
    expect(data).toHaveProperty('period_days')
    expect(data).toHaveProperty('session_count')
    expect(data).toHaveProperty('total_tokens')
    expect(data).toHaveProperty('total_est_cost_usd')
    expect(data).toHaveProperty('by_agent')
    expect(data).toHaveProperty('by_model')
    expect(data).toHaveProperty('top_tools')
    expect(Array.isArray(data.by_agent)).toBe(true)
    expect(Array.isArray(data.top_tools)).toBe(true)
  })

  test('/api/entire-states returns array', async ({ request }) => {
    const res = await request.get('/api/entire-states')
    expect(res.ok()).toBeTruthy()
    const data = await res.json()
    expect(Array.isArray(data)).toBe(true)
  })
})
