// SolverRunner: Manages Web Worker execution of WASM solvers

import type { GridMap, Coordinate, Path, Solution, SolverResult, SolverStats } from '$lib/types';
import { validateSolution, type ValidationResult } from '$lib/validation';

export interface SolverRunnerConfig {
	/** Timeout in milliseconds (default: 30000) */
	timeoutMs?: number;
	/** Max memory in MB (informational, enforced by browser) */
	maxMemoryMb?: number;
}

export type WorkerMessageType = 'init' | 'solve' | 'info';

export interface WorkerRequest {
    id: number;
    type: WorkerMessageType;
    payload?: {
        mode?: 'bindgen' | 'component';
        wasmUrl?: string;
        jsUrl?: string;
        mapData?: Uint8Array;
        width?: number;
        height?: number;
        starts?: Uint32Array;
        goals?: Uint32Array;
        algorithm?: string;
        heuristic?: string;
    };
}

export interface WorkerResponse {
	id: number;
	success: boolean;
	data?: {
		info?: string;
		paths?: Path[];
		cost?: number;
		nodesExpanded?: number;
		timeUs?: number;
	};
	error?: string;
}

/**
 * Manages solver execution in a Web Worker with timeout support.
 */
export class SolverRunner {
	private worker: Worker | null = null;
	private config: Required<SolverRunnerConfig>;
	private requestId = 0;
	private pendingRequests = new Map<
		number,
		{ resolve: (value: WorkerResponse) => void; reject: (error: Error) => void }
	>();
	private running = false;

	constructor(config: SolverRunnerConfig = {}) {
		this.config = {
			timeoutMs: config.timeoutMs ?? 30000,
			maxMemoryMb: config.maxMemoryMb ?? 512
		};
	}

	/**
	 * Initialize the worker with a WASM solver module.
	 * @param wasmUrl URL to the .wasm file
	 * @param jsUrl URL to the JS glue file (wasm-bindgen output)
	 */
	async init(wasmUrl: string, jsUrl: string): Promise<string> {
		this.terminate(); // Clean up any existing worker

		this.worker = new Worker(new URL('./solver.worker.ts', import.meta.url), {
			type: 'module'
		});

		this.worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
			const { id, success, data, error } = event.data;
			const pending = this.pendingRequests.get(id);
			if (pending) {
				this.pendingRequests.delete(id);
				if (success) {
					pending.resolve(event.data);
				} else {
					pending.reject(new Error(error ?? 'Unknown error'));
				}
			}
		};

		this.worker.onerror = (event) => {
			console.error('Worker error:', event);
			// Reject all pending requests
			for (const [id, pending] of this.pendingRequests) {
				pending.reject(new Error('Worker error: ' + event.message));
				this.pendingRequests.delete(id);
			}
		};

		const response = await this.sendRequest('init', { wasmUrl, jsUrl });
		return response.data?.info ?? 'Unknown solver';
	}

	/**
	 * Initialize the solver with a custom WASM component (user upload).
	 * Custom WASM components are no longer supported in the browser on Cloudflare.
	 * Users should upload WASM to the backend API for verification instead.
	 * @param wasmBuffer Raw bytes of the .wasm component
	 */
	async initComponent(wasmBuffer: ArrayBuffer): Promise<string> {
		throw new Error(
			'Custom WASM solvers cannot run in the browser on Cloudflare Pages. ' +
			'Please use the backend verification API to test your solver. ' +
			'The backend provides deterministic instruction counting and full WASM Component Model support.'
		);
	}

	/**
	 * Run the solver on a MAPF instance.
	 * Validates the solution to ensure it follows MAPF rules:
	 * - Cardinal moves only (no diagonals)
	 * - No collisions between agents
	 * - Paths stay on passable cells
	 */
	async solve(
		map: GridMap, 
		starts: Coordinate[], 
		goals: Coordinate[],
		options: { algorithm?: string; heuristic?: string } = {}
	): Promise<SolverResult> {
		if (!this.worker) {
			return { status: 'error', error: 'Solver not initialized' };
		}

		// Validate inputs
		try {
			SolverRunner.validateInputs(map, starts, goals);
		} catch (e) {
			return { status: 'error', error: e instanceof Error ? e.message : 'Invalid input' };
		}

		this.running = true;
		const startTime = performance.now();

		try {
			const response = await this.sendRequestWithTimeout('solve', {
				mapData: map.tiles,
				width: map.width,
				height: map.height,
				starts: SolverRunner.flattenCoordinates(starts),
				goals: SolverRunner.flattenCoordinates(goals),
				algorithm: options.algorithm,
				heuristic: options.heuristic
			});

			const endTime = performance.now();
			// Paths come pre-parsed from worker
			const paths = response.data?.paths ?? [];

			const solution: Solution = {
				paths,
				cost: response.data?.cost ?? 0
			};

			const stats: SolverStats = {
				nodesExpanded: response.data?.nodesExpanded ?? 0,
				timeUs: Math.round((endTime - startTime) * 1000)
			};

			// Validate the solution
			const validation = validateSolution(solution, map, starts, goals);
			if (!validation.valid) {
				// Solution violates MAPF rules
				const firstError = validation.errors[0];
				const errorSummary = validation.errors.length > 1
					? `${firstError.details} (and ${validation.errors.length - 1} more issues)`
					: firstError.details;
				return {
					status: 'error',
					error: `Invalid solution: ${errorSummary}`,
					solution, // Include solution for debugging
					stats
				};
			}

			return { status: 'success', solution, stats };
		} catch (e) {
			if (e instanceof Error && e.message.includes('timeout')) {
				return { status: 'timeout', error: 'Solver timed out' };
			}
			return { status: 'error', error: e instanceof Error ? e.message : 'Solve failed' };
		} finally {
			this.running = false;
		}
	}

	/**
	 * Check if solver is currently running.
	 */
	isRunning(): boolean {
		return this.running;
	}

	/**
	 * Terminate the worker.
	 */
	terminate(): void {
		if (this.worker) {
			this.worker.terminate();
			this.worker = null;
		}
		// Reject all pending requests
		for (const [, pending] of this.pendingRequests) {
			pending.reject(new Error('Worker terminated'));
		}
		this.pendingRequests.clear();
		this.running = false;
	}

	private sendRequest(
		type: WorkerMessageType,
		payload?: WorkerRequest['payload']
	): Promise<WorkerResponse> {
		return new Promise((resolve, reject) => {
			if (!this.worker) {
				reject(new Error('Worker not initialized'));
				return;
			}

			const id = ++this.requestId;
			this.pendingRequests.set(id, { resolve, reject });

			const request: WorkerRequest = { id, type, payload };
			this.worker.postMessage(request);
		});
	}

	private sendRequestWithTimeout(
		type: WorkerMessageType,
		payload?: WorkerRequest['payload']
	): Promise<WorkerResponse> {
		return new Promise((resolve, reject) => {
			const timeout = setTimeout(() => {
				this.pendingRequests.delete(this.requestId);
				// Terminate and recreate worker on timeout
				this.terminate();
				reject(new Error('Request timeout'));
			}, this.config.timeoutMs);

			this.sendRequest(type, payload)
				.then((response) => {
					clearTimeout(timeout);
					resolve(response);
				})
				.catch((error) => {
					clearTimeout(timeout);
					reject(error);
				});
		});
	}

	// ─────────────────────────────────────────────────────────────────────────────
	// Static utility methods
	// ─────────────────────────────────────────────────────────────────────────────

	/**
	 * Flatten coordinates to [x1, y1, x2, y2, ...] format.
	 */
	static flattenCoordinates(coords: Coordinate[]): Uint32Array {
		const flat = new Uint32Array(coords.length * 2);
		for (let i = 0; i < coords.length; i++) {
			flat[i * 2] = coords[i].x;
			flat[i * 2 + 1] = coords[i].y;
		}
		return flat;
	}

	/**
	 * Validate solver inputs.
	 */
	static validateInputs(map: GridMap, starts: Coordinate[], goals: Coordinate[]): void {
		if (starts.length !== goals.length) {
			throw new Error('Starts and goals must have the same number of agents');
		}

		for (let i = 0; i < starts.length; i++) {
			const start = starts[i];
			const goal = goals[i];

			// Bounds check
			if (start.x >= map.width || start.y >= map.height) {
				throw new Error(`Agent ${i} start (${start.x},${start.y}) out of bounds`);
			}
			if (goal.x >= map.width || goal.y >= map.height) {
				throw new Error(`Agent ${i} goal (${goal.x},${goal.y}) out of bounds`);
			}

			// Passability check
			const startIdx = start.y * map.width + start.x;
			const goalIdx = goal.y * map.width + goal.x;
			if (map.tiles[startIdx] !== 1) {
				throw new Error(`Agent ${i} start is not passable`);
			}
			if (map.tiles[goalIdx] !== 1) {
				throw new Error(`Agent ${i} goal is not passable`);
			}
		}
	}
}
