// Tests for SolverRunner service

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { SolverRunner, type SolverRunnerConfig } from './runner';
import type { GridMap, Coordinate } from '$lib/types';

// Mock worker for testing (actual worker tests need browser environment)
describe('SolverRunner', () => {
	let runner: SolverRunner;

	const mockMap: GridMap = {
		width: 8,
		height: 8,
		tiles: new Uint8Array(64).fill(1) // All passable
	};

	const mockStarts: Coordinate[] = [{ x: 0, y: 0 }];
	const mockGoals: Coordinate[] = [{ x: 7, y: 7 }];

	beforeEach(() => {
		runner = new SolverRunner();
	});

	afterEach(() => {
		runner.terminate();
	});

	it('creates runner with default config', () => {
		expect(runner).toBeDefined();
		expect(runner.isRunning()).toBe(false);
	});

	it('creates runner with custom config', () => {
		const config: SolverRunnerConfig = {
			timeoutMs: 5000,
			maxMemoryMb: 256
		};
		const customRunner = new SolverRunner(config);
		expect(customRunner).toBeDefined();
		customRunner.terminate();
	});

	it('converts coordinates to flat arrays', () => {
		const coords: Coordinate[] = [
			{ x: 1, y: 2 },
			{ x: 3, y: 4 }
		];
		const flat = SolverRunner.flattenCoordinates(coords);
		expect(flat).toEqual(new Uint32Array([1, 2, 3, 4]));
	});

	// Note: parsePaths was moved to the worker - parsing now happens there
	// The runner receives pre-parsed paths from the worker

	it('validates inputs', () => {
		// Mismatched starts/goals
		expect(() => {
			SolverRunner.validateInputs(mockMap, mockStarts, []);
		}).toThrow(/same number/);

		// Start out of bounds
		expect(() => {
			SolverRunner.validateInputs(mockMap, [{ x: 100, y: 0 }], [{ x: 0, y: 0 }]);
		}).toThrow(/bounds/);

		// Goal on blocked cell
		const blockedMap = { ...mockMap, tiles: new Uint8Array(64).fill(0) };
		expect(() => {
			SolverRunner.validateInputs(blockedMap, mockStarts, mockGoals);
		}).toThrow(/passable/);
	});

	it('accepts valid inputs', () => {
		expect(() => {
			SolverRunner.validateInputs(mockMap, mockStarts, mockGoals);
		}).not.toThrow();
	});
});
