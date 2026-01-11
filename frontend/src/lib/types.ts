// Core MAPF types matching the Rust/WIT definitions

/** A 2D coordinate on the grid */
export interface Coordinate {
	x: number;
	y: number;
}

/** A path from start to goal */
export interface Path {
	steps: Coordinate[];
}

/** Result of solving a MAPF instance */
export interface Solution {
	paths: Path[];
	cost: number;
}

/** Statistics from a solver run */
export interface SolverStats {
	nodesExpanded: number;
	timeUs: number;
}

/** A parsed MovingAI map */
export interface GridMap {
	width: number;
	height: number;
	/** Row-major tiles: 1 = passable, 0 = blocked */
	tiles: Uint8Array;
}

/** A scenario entry (agent task) */
export interface ScenarioEntry {
	bucket: number;
	mapName: string;
	mapWidth: number;
	mapHeight: number;
	startX: number;
	startY: number;
	goalX: number;
	goalY: number;
	optimalLength: number;
}

/** A parsed MovingAI scenario */
export interface Scenario {
	version: number;
	entries: ScenarioEntry[];
}

/** Solver status during execution */
export type SolverStatus = 'idle' | 'running' | 'success' | 'error' | 'timeout';

/** Result from running a solver */
export interface SolverResult {
	status: SolverStatus;
	solution?: Solution;
	stats?: SolverStats;
	error?: string;
}
