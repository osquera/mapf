import { test, expect } from '@playwright/test';

test.describe('Arena Page', () => {
	test('loads with sample map visible', async ({ page }) => {
		await page.goto('/arena');

		// Wait for page to load and take screenshot for debugging
		await page.waitForTimeout(2000);
		await page.screenshot({ path: 'test-results/arena-screenshot.png' });

		// Log page content for debugging
		const content = await page.content();
		console.log('Page content snippet:', content.substring(0, 2000));

		// Should show the Arena heading
		await expect(page.locator('h1')).toContainText('Arena');

		// Should have the map viewer with tiles
		await expect(page.locator('.map-viewer')).toBeVisible();

		// Should show map info
		await expect(page.getByText(/Size:/)).toBeVisible();
		await expect(page.getByText('Agents: 2')).toBeVisible();
	});

	test('solver initializes and becomes ready', async ({ page }) => {
		// Listen for all console messages
		const consoleMessages: string[] = [];
		page.on('console', (msg) => {
			const text = `[${msg.type()}]: ${msg.text()}`;
			consoleMessages.push(text);
			console.log(text);
		});
		
		// Also catch page errors
		page.on('pageerror', (error) => {
			console.log('Page error:', error.message);
		});

		await page.goto('/arena');

		// Wait a bit for solver initialization
		await page.waitForTimeout(5000);
		
		// Get the button text to see current state
		const runButton = page.getByRole('button', { name: /Run Solver|Loading|Solving/ });
		const buttonText = await runButton.textContent();
		console.log('Button text:', buttonText);
		
		// Check for error text on page
		const errorDiv = page.locator('.error');
		const hasError = await errorDiv.count() > 0;
		if (hasError) {
			const errorText = await errorDiv.textContent();
			console.log('Error on page:', errorText);
		}

		// Log all console messages
		console.log('All console messages:', consoleMessages);

		// Wait up to 15s for solver to initialize
		await expect(runButton).toBeEnabled({ timeout: 15000 });

		// Should contain Run Solver text when ready
		await expect(runButton).toContainText('Run Solver');
	});

	test('can run solver and see results', async ({ page }) => {
		// Capture console for debugging
		page.on('console', (msg) => {
			console.log(`Browser ${msg.type()}: ${msg.text()}`);
		});

		await page.goto('/arena');

		// Wait for solver to be ready
		const runButton = page.getByRole('button', { name: /Run Solver|Loading|Solving/ });
		await expect(runButton).toBeEnabled({ timeout: 15000 });

		// Click Run Solver
		await runButton.click();

		// Should show "Solving..." briefly, then results
		await expect(page.getByText('Results')).toBeVisible({ timeout: 15000 });

		// Should show solution found (multi-agent with collision avoidance)
		await expect(page.getByText(/Solution found/)).toBeVisible({ timeout: 5000 });
		
		// Should show timing info
		await expect(page.getByText(/Time:/)).toBeVisible();
		
		// Should show paths info for both agents
		await expect(page.getByText(/Paths: 2/)).toBeVisible();
	});

	test('file upload for map works', async ({ page }) => {
		await page.goto('/arena');

		// The map file input should exist
		const mapInput = page.locator('input[type="file"][accept=".map"]');
		await expect(mapInput).toBeVisible();
	});

	test('file upload for scenario works', async ({ page }) => {
		await page.goto('/arena');

		// The scenario file input should exist
		const scenInput = page.locator('input[type="file"][accept=".scen"]');
		await expect(scenInput).toBeVisible();
	});

	test('Load Sample Map button resets state', async ({ page }) => {
		await page.goto('/arena');

		// Click Load Sample Map
		await page.locator('button', { hasText: 'Load Sample Map' }).click();

		// Should show the sample map info
		await expect(page.getByText(/Size: 16/)).toBeVisible();
		await expect(page.getByText('Agents: 2')).toBeVisible();
	});
});
