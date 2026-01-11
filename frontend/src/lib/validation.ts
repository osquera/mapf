// Path validation for MAPF solutions
// Ensures solvers follow the rules: cardinal moves only, no collisions

import type { Coordinate, Path, Solution, GridMap } from './types';

/** Validation error types */
export type ValidationErrorType =
	| 'diagonal_move'
	| 'out_of_bounds'
	| 'blocked_cell'
	| 'invalid_start'
	| 'invalid_goal'
	| 'vertex_collision'
	| 'edge_collision'
	| 'empty_path';

/** A validation error with details */
export interface ValidationError {
	type: ValidationErrorType;
	agentIndex: number;
	timestep?: number;
	details: string;
}

/** Result of validating a solution */
export interface ValidationResult {
	valid: boolean;
	errors: ValidationError[];
}

/**
 * Check if a move is cardinal (N/S/E/W only, no diagonals).
 */
export function isCardinalMove(from: Coordinate, to: Coordinate): boolean {
	const dx = Math.abs(to.x - from.x);
	const dy = Math.abs(to.y - from.y);
	// Valid: (1,0), (0,1), or (0,0) for wait
	return (dx === 1 && dy === 0) || (dx === 0 && dy === 1) || (dx === 0 && dy === 0);
}

/**
 * Validate that a single path uses only cardinal moves.
 */
export function validatePathCardinal(path: Path, agentIndex: number): ValidationError[] {
	const errors: ValidationError[] = [];

	if (path.steps.length === 0) {
		errors.push({
			type: 'empty_path',
			agentIndex,
			details: `Agent ${agentIndex} has empty path`
		});
		return errors;
	}

	for (let t = 0; t < path.steps.length - 1; t++) {
		const from = path.steps[t];
		const to = path.steps[t + 1];

		if (!isCardinalMove(from, to)) {
			errors.push({
				type: 'diagonal_move',
				agentIndex,
				timestep: t,
				details: `Agent ${agentIndex} made diagonal move from (${from.x},${from.y}) to (${to.x},${to.y}) at timestep ${t}`
			});
		}
	}

	return errors;
}

/**
 * Validate that a path stays within map bounds and on passable cells.
 */
export function validatePathOnMap(
	path: Path,
	agentIndex: number,
	map: GridMap
): ValidationError[] {
	const errors: ValidationError[] = [];

	for (let t = 0; t < path.steps.length; t++) {
		const pos = path.steps[t];

		// Bounds check
		if (pos.x < 0 || pos.x >= map.width || pos.y < 0 || pos.y >= map.height) {
			errors.push({
				type: 'out_of_bounds',
				agentIndex,
				timestep: t,
				details: `Agent ${agentIndex} at (${pos.x},${pos.y}) is out of bounds at timestep ${t}`
			});
			continue;
		}

		// Passable check
		const idx = pos.y * map.width + pos.x;
		if (map.tiles[idx] === 0) {
			errors.push({
				type: 'blocked_cell',
				agentIndex,
				timestep: t,
				details: `Agent ${agentIndex} at (${pos.x},${pos.y}) is on blocked cell at timestep ${t}`
			});
		}
	}

	return errors;
}

/**
 * Validate that paths don't have vertex collisions (two agents at same cell at same time).
 */
export function validateNoVertexCollisions(paths: Path[]): ValidationError[] {
	const errors: ValidationError[] = [];

	// Find max timestep
	const maxT = Math.max(...paths.map((p) => p.steps.length));

	for (let t = 0; t < maxT; t++) {
		// Map of position -> agent index at this timestep
		const occupied = new Map<string, number>();

		for (let agent = 0; agent < paths.length; agent++) {
			const path = paths[agent];
			// If path ended, agent stays at last position
			const pos = t < path.steps.length ? path.steps[t] : path.steps[path.steps.length - 1];
			const key = `${pos.x},${pos.y}`;

			if (occupied.has(key)) {
				const otherAgent = occupied.get(key)!;
				errors.push({
					type: 'vertex_collision',
					agentIndex: agent,
					timestep: t,
					details: `Agents ${otherAgent} and ${agent} collide at (${pos.x},${pos.y}) at timestep ${t}`
				});
			} else {
				occupied.set(key, agent);
			}
		}
	}

	return errors;
}

/**
 * Validate that paths don't have edge collisions (two agents swapping positions).
 */
export function validateNoEdgeCollisions(paths: Path[]): ValidationError[] {
	const errors: ValidationError[] = [];

	// Find max timestep
	const maxT = Math.max(...paths.map((p) => p.steps.length));

	for (let t = 0; t < maxT - 1; t++) {
		for (let i = 0; i < paths.length; i++) {
			for (let j = i + 1; j < paths.length; j++) {
				const pathI = paths[i];
				const pathJ = paths[j];

				// Get positions at t and t+1 for both agents
				const posI_t = t < pathI.steps.length ? pathI.steps[t] : pathI.steps[pathI.steps.length - 1];
				const posI_t1 = t + 1 < pathI.steps.length ? pathI.steps[t + 1] : pathI.steps[pathI.steps.length - 1];
				const posJ_t = t < pathJ.steps.length ? pathJ.steps[t] : pathJ.steps[pathJ.steps.length - 1];
				const posJ_t1 = t + 1 < pathJ.steps.length ? pathJ.steps[t + 1] : pathJ.steps[pathJ.steps.length - 1];

				// Check if they swap (edge collision)
				if (
					posI_t.x === posJ_t1.x &&
					posI_t.y === posJ_t1.y &&
					posJ_t.x === posI_t1.x &&
					posJ_t.y === posI_t1.y
				) {
					errors.push({
						type: 'edge_collision',
						agentIndex: i,
						timestep: t,
						details: `Agents ${i} and ${j} swap positions between timesteps ${t} and ${t + 1}`
					});
				}
			}
		}
	}

	return errors;
}

/**
 * Validate that paths start and end at the correct positions.
 */
export function validateStartsAndGoals(
	paths: Path[],
	starts: Coordinate[],
	goals: Coordinate[]
): ValidationError[] {
	const errors: ValidationError[] = [];

	for (let i = 0; i < paths.length; i++) {
		const path = paths[i];
		if (path.steps.length === 0) continue;

		const pathStart = path.steps[0];
		const pathEnd = path.steps[path.steps.length - 1];
		const expectedStart = starts[i];
		const expectedGoal = goals[i];

		if (pathStart.x !== expectedStart.x || pathStart.y !== expectedStart.y) {
			errors.push({
				type: 'invalid_start',
				agentIndex: i,
				timestep: 0,
				details: `Agent ${i} path starts at (${pathStart.x},${pathStart.y}) but should start at (${expectedStart.x},${expectedStart.y})`
			});
		}

		if (pathEnd.x !== expectedGoal.x || pathEnd.y !== expectedGoal.y) {
			errors.push({
				type: 'invalid_goal',
				agentIndex: i,
				timestep: path.steps.length - 1,
				details: `Agent ${i} path ends at (${pathEnd.x},${pathEnd.y}) but should end at (${expectedGoal.x},${expectedGoal.y})`
			});
		}
	}

	return errors;
}

/**
 * Fully validate a MAPF solution.
 *
 * Checks:
 * 1. All moves are cardinal (N/S/E/W) or wait
 * 2. All positions are within bounds and on passable cells
 * 3. Paths start and end at correct positions
 * 4. No vertex collisions (two agents at same cell)
 * 5. No edge collisions (two agents swapping)
 */
export function validateSolution(
	solution: Solution,
	map: GridMap,
	starts: Coordinate[],
	goals: Coordinate[]
): ValidationResult {
	const errors: ValidationError[] = [];

	// Validate each path individually
	for (let i = 0; i < solution.paths.length; i++) {
		const path = solution.paths[i];
		errors.push(...validatePathCardinal(path, i));
		errors.push(...validatePathOnMap(path, i, map));
	}

	// Validate starts and goals
	errors.push(...validateStartsAndGoals(solution.paths, starts, goals));

	// Validate collisions between agents
	if (solution.paths.length > 1) {
		errors.push(...validateNoVertexCollisions(solution.paths));
		errors.push(...validateNoEdgeCollisions(solution.paths));
	}

	return {
		valid: errors.length === 0,
		errors
	};
}
