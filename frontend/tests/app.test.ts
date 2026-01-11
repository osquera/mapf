import { test, expect } from '@playwright/test';

test.describe('Home page', () => {
	test('has title', async ({ page }) => {
		await page.goto('/');
		await expect(page).toHaveTitle(/MAPF Arena/);
	});

	test('shows hero section', async ({ page }) => {
		await page.goto('/');
		await expect(page.locator('h1')).toContainText('MAPF Arena');
	});

	test('has navigation links', async ({ page }) => {
		await page.goto('/');
		await expect(page.locator('nav a[href="/"]')).toBeVisible();
		await expect(page.locator('nav a[href="/arena"]')).toBeVisible();
		await expect(page.locator('nav a[href="/leaderboard"]')).toBeVisible();
		await expect(page.locator('nav a[href="/upload"]')).toBeVisible();
	});

	test('can navigate to arena', async ({ page }) => {
		await page.goto('/');
		await page.click('nav a[href="/arena"]');
		await expect(page.locator('h1')).toContainText('Arena');
	});
});

test.describe('Arena page', () => {
	test('shows map viewer', async ({ page }) => {
		await page.goto('/arena');
		await expect(page.locator('canvas')).toBeVisible();
	});

	test('shows map info', async ({ page }) => {
		await page.goto('/arena');
		await expect(page.locator('text=Map Info')).toBeVisible();
	});
});

test.describe('Upload page', () => {
	test('shows upload form', async ({ page }) => {
		await page.goto('/upload');
		await expect(page.locator('h1')).toContainText('Upload Solver');
		await expect(page.locator('input[type="file"]')).toBeVisible();
	});

	test('shows requirements', async ({ page }) => {
		await page.goto('/upload');
		await expect(page.locator('text=Solver Requirements')).toBeVisible();
	});
});

test.describe('Leaderboard page', () => {
	test('shows rankings table', async ({ page }) => {
		await page.goto('/leaderboard');
		await expect(page.locator('table')).toBeVisible();
		await expect(page.locator('th:has-text("Rank")')).toBeVisible();
	});
});
