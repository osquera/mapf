// Tests for MAPF solution validation

import { describe, it, expect } from 'vitest';
import {
	isCardinalMove,
	validatePathCardinal,
	validatePathOnMap,
	validateNoVertexCollisions,
	validateNoEdgeCollisions,
	validateStartsAndGoals,
	validateSolution
} from './validation';
import type { Path, GridMap, Coordinate, Solution } from './types';

// ─────────────────────────────────────────────────────────────────────────────
// Cardinal Move Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('isCardinalMove', () => {
	it('accepts North move', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 5, y: 4 })).toBe(true);
	});

	it('accepts South move', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 5, y: 6 })).toBe(true);
	});

	it('accepts East move', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 6, y: 5 })).toBe(true);
	});

	it('accepts West move', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 4, y: 5 })).toBe(true);
	});

	it('accepts wait (no move)', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 5, y: 5 })).toBe(true);
	});

	it('rejects diagonal move NE', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 6, y: 4 })).toBe(false);
	});

	it('rejects diagonal move SE', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 6, y: 6 })).toBe(false);
	});

	it('rejects diagonal move SW', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 4, y: 6 })).toBe(false);
	});

	it('rejects diagonal move NW', () => {
		expect(isCardinalMove({ x: 5, y: 5 }, { x: 4, y: 4 })).toBe(false);
	});

	it('rejects teleport (jump)', () => {
		expect(isCardinalMove({ x: 0, y: 0 }, { x: 5, y: 5 })).toBe(false);
	});
});

// ─────────────────────────────────────────────────────────────────────────────
// Path Validation Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('validatePathCardinal', () => {
	it('accepts valid cardinal path', () => {
		const path: Path = {
			steps: [
				{ x: 0, y: 0 },
				{ x: 1, y: 0 }, // East
				{ x: 1, y: 1 }, // South
				{ x: 2, y: 1 } // East
			]
		};
		const errors = validatePathCardinal(path, 0);
		expect(errors).toHaveLength(0);
	});

	it('rejects path with diagonal move', () => {
		const path: Path = {
			steps: [
				{ x: 0, y: 0 },
				{ x: 1, y: 1 } // Diagonal!
			]
		};
		const errors = validatePathCardinal(path, 0);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('diagonal_move');
		expect(errors[0].agentIndex).toBe(0);
		expect(errors[0].timestep).toBe(0);
	});

	it('reports multiple diagonal moves', () => {
		const path: Path = {
			steps: [
				{ x: 0, y: 0 },
				{ x: 1, y: 1 }, // Diagonal
				{ x: 2, y: 2 } // Diagonal
			]
		};
		const errors = validatePathCardinal(path, 1);
		expect(errors).toHaveLength(2);
		expect(errors[0].agentIndex).toBe(1);
		expect(errors[1].agentIndex).toBe(1);
	});

	it('reports empty path', () => {
		const path: Path = { steps: [] };
		const errors = validatePathCardinal(path, 0);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('empty_path');
	});
});

describe('validatePathOnMap', () => {
	const map: GridMap = {
		width: 4,
		height: 4,
		tiles: new Uint8Array([
			1, 1, 1, 1,
			1, 0, 0, 1, // blocked in middle
			1, 1, 1, 1,
			1, 1, 1, 1
		])
	};

	it('accepts path on passable cells', () => {
		const path: Path = {
			steps: [
				{ x: 0, y: 0 },
				{ x: 0, y: 1 },
				{ x: 0, y: 2 }
			]
		};
		const errors = validatePathOnMap(path, 0, map);
		expect(errors).toHaveLength(0);
	});

	it('rejects path through blocked cell', () => {
		const path: Path = {
			steps: [
				{ x: 0, y: 1 },
				{ x: 1, y: 1 } // blocked!
			]
		};
		const errors = validatePathOnMap(path, 0, map);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('blocked_cell');
	});

	it('rejects path out of bounds', () => {
		const path: Path = {
			steps: [
				{ x: 3, y: 3 },
				{ x: 4, y: 3 } // out of bounds!
			]
		};
		const errors = validatePathOnMap(path, 0, map);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('out_of_bounds');
	});
});

// ─────────────────────────────────────────────────────────────────────────────
// Collision Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('validateNoVertexCollisions', () => {
	it('accepts paths with no collisions', () => {
		const paths: Path[] = [
			{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 2, y: 0 }] },
			{ steps: [{ x: 0, y: 2 }, { x: 1, y: 2 }, { x: 2, y: 2 }] }
		];
		const errors = validateNoVertexCollisions(paths);
		expect(errors).toHaveLength(0);
	});

	it('detects vertex collision', () => {
		const paths: Path[] = [
			{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }] },
			{ steps: [{ x: 2, y: 0 }, { x: 1, y: 0 }] } // Both at (1,0) at t=1
		];
		const errors = validateNoVertexCollisions(paths);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('vertex_collision');
		expect(errors[0].timestep).toBe(1);
	});
});

describe('validateNoEdgeCollisions', () => {
	it('accepts paths with no edge collisions', () => {
		const paths: Path[] = [
			{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }] },
			{ steps: [{ x: 0, y: 1 }, { x: 1, y: 1 }] }
		];
		const errors = validateNoEdgeCollisions(paths);
		expect(errors).toHaveLength(0);
	});

	it('detects edge collision (swap)', () => {
		const paths: Path[] = [
			{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }] }, // Moving right
			{ steps: [{ x: 1, y: 0 }, { x: 0, y: 0 }] } // Moving left - SWAP!
		];
		const errors = validateNoEdgeCollisions(paths);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('edge_collision');
	});
});

// ─────────────────────────────────────────────────────────────────────────────
// Start/Goal Validation
// ─────────────────────────────────────────────────────────────────────────────

describe('validateStartsAndGoals', () => {
	const starts: Coordinate[] = [{ x: 0, y: 0 }, { x: 3, y: 0 }];
	const goals: Coordinate[] = [{ x: 3, y: 3 }, { x: 0, y: 3 }];

	it('accepts paths with correct starts and goals', () => {
		const paths: Path[] = [
			{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 2, y: 0 }, { x: 3, y: 0 }, { x: 3, y: 1 }, { x: 3, y: 2 }, { x: 3, y: 3 }] },
			{ steps: [{ x: 3, y: 0 }, { x: 2, y: 0 }, { x: 1, y: 0 }, { x: 0, y: 0 }, { x: 0, y: 1 }, { x: 0, y: 2 }, { x: 0, y: 3 }] }
		];
		const errors = validateStartsAndGoals(paths, starts, goals);
		expect(errors).toHaveLength(0);
	});

	it('rejects path with wrong start', () => {
		const paths: Path[] = [
			{ steps: [{ x: 1, y: 1 }, { x: 2, y: 1 }] } // Wrong start!
		];
		const errors = validateStartsAndGoals(paths, [{ x: 0, y: 0 }], [{ x: 2, y: 1 }]);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('invalid_start');
	});

	it('rejects path with wrong goal', () => {
		const paths: Path[] = [
			{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }] } // Wrong goal!
		];
		const errors = validateStartsAndGoals(paths, [{ x: 0, y: 0 }], [{ x: 5, y: 5 }]);
		expect(errors).toHaveLength(1);
		expect(errors[0].type).toBe('invalid_goal');
	});
});

// ─────────────────────────────────────────────────────────────────────────────
// Full Solution Validation
// ─────────────────────────────────────────────────────────────────────────────

describe('validateSolution', () => {
	const map: GridMap = {
		width: 4,
		height: 4,
		tiles: new Uint8Array(16).fill(1) // All passable
	};

	it('validates a correct solution', () => {
		const solution: Solution = {
			paths: [
				{ steps: [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 2, y: 0 }] },
				{ steps: [{ x: 0, y: 2 }, { x: 1, y: 2 }, { x: 2, y: 2 }] }
			],
			cost: 4
		};
		const starts = [{ x: 0, y: 0 }, { x: 0, y: 2 }];
		const goals = [{ x: 2, y: 0 }, { x: 2, y: 2 }];

		const result = validateSolution(solution, map, starts, goals);
		expect(result.valid).toBe(true);
		expect(result.errors).toHaveLength(0);
	});

	it('detects multiple issues in invalid solution', () => {
		const solution: Solution = {
			paths: [
				{ steps: [{ x: 0, y: 0 }, { x: 1, y: 1 }] }, // Diagonal move!
				{ steps: [{ x: 3, y: 0 }, { x: 1, y: 1 }] } // Diagonal + collision at (1,1)!
			],
			cost: 2
		};
		const starts = [{ x: 0, y: 0 }, { x: 3, y: 0 }];
		const goals = [{ x: 1, y: 1 }, { x: 1, y: 1 }]; // Both going to same goal

		const result = validateSolution(solution, map, starts, goals);
		expect(result.valid).toBe(false);
		// Should have: 2 diagonal moves + 1 vertex collision
		expect(result.errors.length).toBeGreaterThanOrEqual(3);
	});
});
