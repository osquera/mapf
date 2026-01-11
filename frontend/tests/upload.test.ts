import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';

test.describe('Solver Upload', () => {
    test('can upload and run custom WASM solver', async ({ page }) => {
        // Ensure the test component exists
        const wasmRelativePath = 'static/test-solver.wasm';
        const wasmPath = path.resolve(process.cwd(), wasmRelativePath);
        
        if (!fs.existsSync(wasmPath)) {
            // Fallback for different CWD (e.g. if running from frontend dir)
            const altPath = path.resolve(process.cwd(), 'frontend/static/test-solver.wasm');
            if (!fs.existsSync(altPath)) {
                throw new Error(`Test solver not found at ${wasmPath} or ${altPath}. Did you run 'task build:test-component'?`);
            }
        }

        await page.goto('/arena');

        // Locate the file input (it might be hidden or styled, so we use locator pointing to input[type=file])
        const fileInput = page.locator('input[type="file"]');
        
        // Upload the WASM file
        await fileInput.setInputFiles(wasmPath);

        // Wait for the solver name to update, indicating successful initialization
        // The test component returns "Test Component Solver v0.1"
        // Avoid strict mode violation by scoping to the specific status area or being more precise
        // The UI shows: "✅ Loaded: Test Component Solver v0.1"
        await expect(page.locator('.solver-status')).toContainText('Test Component Solver v0.1');

        // Run the solver
        // The button text updates to "Run [Solver Name]"
        const runButton = page.getByRole('button', { name: /Run/ });
        await expect(runButton).toBeEnabled();
        await runButton.click();

        // The test component executes a "wait" strategy which is invalid for the goal
        // expecting the validation error "Invalid solution"
        await expect(page.getByText(/Invalid solution/)).toBeVisible({ timeout: 10000 });
        
        // Also verify specific error detail if possible, confirming it ran logic
        await expect(page.getByText(/Agent 0 path ends at/)).toBeVisible();
    });
});
