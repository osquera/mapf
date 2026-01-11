// Web Worker for running WASM solvers in isolation
// Note: This worker cannot use $lib imports - it must be self-contained

interface Coordinate {
	x: number;
	y: number;
}

interface WorkerRequest {
	id: number;
	type: 'init' | 'solve' | 'info';
	payload?: {
		mode?: 'bindgen' | 'component';
		wasmUrl?: string;
		jsUrl?: string;
		mapData?: Uint8Array;
		width?: number;
		height?: number;
		starts?: Uint32Array;  // Flattened: [x1, y1, x2, y2, ...]
		goals?: Uint32Array;   // Flattened: [x1, y1, x2, y2, ...]
		algorithm?: string;
		heuristic?: string;
	};
}

interface WorkerResponse {
	id: number;
	success: boolean;
	data?: {
		info?: string;
		paths?: { steps: Coordinate[] }[];
		cost?: number;
		nodesExpanded?: number;
		timeUs?: number;
	};
	error?: string;
}

// WASM Solution interface
interface WasmSolution {
	paths: Uint32Array;
	cost: number;
	nodes_expanded: bigint;
	free(): void;
}

// MapfSolver class from WASM (new pattern)
interface MapfSolver {
	solve(starts: Uint32Array, goals: Uint32Array): WasmSolution;
	width: number;
	height: number;
	free(): void;
}

// Constructor for MapfSolver
interface MapfSolverConstructor {
	new (mapData: Uint8Array, width: number, height: number): MapfSolver;
}

// WASM module interface
interface WasmModule {
	MapfSolver?: MapfSolverConstructor; // bindgen
	solve?: (mapData: Uint8Array, width: number, height: number, starts: any, goals: any) => any; // component
	solver_info?: () => string; // bindgen
	info?: () => string; // component
	get_stats?: () => any; // component optional
}

let wasmModule: WasmModule | null = null;
let currentSolver: MapfSolver | null = null;
let currentMapHash: string | null = null;
let solverMode: 'bindgen' | 'component' = 'bindgen';

/**
 * Handle messages from the main thread.
 */
self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const { id, type, payload } = event.data;

	try {
		switch (type) {
			case 'init':
				await handleInit(id, payload?.wasmUrl, payload?.jsUrl, payload?.mode);
				break;
			case 'solve':
				await handleSolve(id, payload);
				break;
			case 'info':
				handleInfo(id);
				break;
			default:
				sendError(id, `Unknown message type: ${type}`);
		}
	} catch (error) {
		sendError(id, error instanceof Error ? error.message : 'Unknown error');
	}
};

/**
 * Initialize the WASM module by fetching and instantiating it.
 */
async function handleInit(id: number, wasmUrl?: string, jsUrl?: string, mode: 'bindgen' | 'component' = 'bindgen'): Promise<void> {
	try {
		if (!jsUrl) {
			throw new Error('jsUrl is required for initialization');
		}

		solverMode = mode;

		// Import the JS glue code that wasm-bindgen generated
		const glue = await import(/* @vite-ignore */ jsUrl);

		if (mode === 'bindgen') {
			// Initialize WASM - if wasmUrl provided, use it; otherwise let it auto-resolve
			if (wasmUrl) {
				await glue.default(wasmUrl);
			} else {
				await glue.default();
			}
			wasmModule = glue as WasmModule;
		} else {
			// Component mode (JCO)
			// Check if we need explicit instantiation (instantiation: 'async')
			if (typeof glue.instantiate === 'function') {
				if (!wasmUrl) throw new Error('WASM URL is required for component instantiation');
				
				// Provide a loader for the core WASM module
				// JCO calls this with the relative path of the core module
				const getCoreModule = async (_path: string) => {
					const response = await fetch(wasmUrl);
					if (!response.ok) throw new Error(`Failed to fetch core WASM from ${wasmUrl}`);
					return WebAssembly.compile(await response.arrayBuffer());
				};

				// Instantiate the component
				// instantiate(getCoreModule, imports, instantiateCore)
				const instance = await glue.instantiate(getCoreModule);
				
				// JCO exports interfaces as named properties, e.g., 'mapf:solver/solver'
				// We need to find the one containing our methods
				
				// Try to find the solver interface
				if (instance.solve) {
					wasmModule = instance as WasmModule;
				} else {
					// Search for nested "solve" function
					const found = Object.values(instance).find((v: any) => typeof v?.solve === 'function');
					if (found) {
						wasmModule = found as WasmModule;
					} else {
						console.error('Available keys:', Object.keys(instance));
						throw new Error('Could not find solver interface (solve function) in component exports');
					}
				}
			} else {
				// Direct ESM exports (instantiation: false/undefined)
				
				if (glue.solve) {
					wasmModule = glue as WasmModule;
				} else {
					// Search for nested "solve" function
					const found = Object.values(glue).find((v: any) => typeof v?.solve === 'function');
					if (found) {
						wasmModule = found as WasmModule;
					} else {
						// Fallback - might start causing errors in solve later if really missing
						console.error('Available keys:', Object.keys(glue));
						throw new Error('Could not find solver interface (solve function) in component exports');
					}
				}
			}
		}

		// Get info
		let infoString = 'Unknown Solver';
		if (mode === 'bindgen' && wasmModule.solver_info) {
			infoString = wasmModule.solver_info();
		} else if (mode === 'component' && wasmModule.info) {
			infoString = wasmModule.info();
		}

		sendResponse(id, { info: infoString });
	} catch (error) {
		throw new Error(`Failed to initialize WASM: ${error instanceof Error ? error.message : error}`);
	}
}

/**
 * Run the solver on a MAPF instance.
 * Uses MapfSolver class for efficient map reuse.
 */
async function handleSolve(
	id: number,
	payload?: WorkerRequest['payload']
): Promise<void> {
	if (!wasmModule) {
		throw new Error('WASM module not initialized');
	}

	if (!payload?.mapData || payload.width === undefined || payload.height === undefined) {
		throw new Error('Missing required solve parameters');
	}

	const { mapData, width, height, starts, goals, algorithm, heuristic } = payload;

	if (!starts || !goals) {
		throw new Error('Missing starts or goals');
	}

	// TODO: Pass algorithm and heuristic to WASM when supported
	// console.log(`Solving with ${algorithm} using ${heuristic}`);

	// starts and goals are already Uint32Array from runner (flattened [x1, y1, x2, y2, ...])

	const startTime = performance.now();

	let paths: { steps: Coordinate[] }[] = [];
	let cost = 0;
	let nodesExpanded = 0;

	if (solverMode === 'bindgen') {
		if (!wasmModule.MapfSolver) throw new Error('MapfSolver class not found in WASM module');
		
		const SolverClass = wasmModule.MapfSolver;
		
		// Create a hash of the map to detect changes
		const mapHash = `${width}x${height}:${mapData.length}`;
		
		// Reuse solver if same map, otherwise create new one
		if (currentMapHash !== mapHash || !currentSolver) {
			if (currentSolver) {
				currentSolver.free();
			}
			currentSolver = new SolverClass(mapData, width, height);
			currentMapHash = mapHash;
		}

		// Call the WASM solver with pre-flattened Uint32Array coordinates
		const solution = currentSolver!.solve(starts, goals);
		
		paths = parseFlattenedPaths(solution.paths);
		
		// Use makespan as cost
		cost = paths.reduce((max, path) => {
			const pathCost = Math.max(0, path.steps.length - 1);
			return Math.max(max, pathCost);
		}, 0);
		
		nodesExpanded = Number(solution.nodes_expanded);
		solution.free();
	} else {
		// Component Mode (JCO)
		// Component expects: solve(mapData, width, height, starts, goals)
		// Coordinates need to be unflattened for Component Model!
		// WIT: solve(..., starts: list<coordinate>, goals: list<coordinate>)
		if (!wasmModule.solve) throw new Error('solve function not found in Component module');

		const startCoords = unflattenCoords(starts);
		const goalCoords = unflattenCoords(goals);

		// JCO handles the Uint8Array -> list<u8> conversion automatically
		const result = wasmModule.solve(mapData, width, height, startCoords, goalCoords);
		
		if (typeof result === 'string') {
			throw new Error(result); // Result is Result<Solution, string>, JCO might map Err to throw or return object
		}
		
		// Check for success/error pattern often used in WIT results if mapped to object
		// But usually `result<T, E>` maps to T on success or throws E (if using some bindings)
		// OR returns { tag: 'ok', val: ... } or { tag: 'err', val: ... }
		
		// Assuming standard JCO output for result:
		// success -> object matching record solution
		// error -> throws exception (default) OR returns variant. 
		// Let's assume JCO standard: returns payload directly on success, throws on error?
		// Actually, result<T,E> often maps to a tagged union in JS: { tag: 'ok', val: ... }
		
		// Let's inspect what JCO produces usually for `result`.
		// It often follows the shape: { tag: 'ok', val: ... } | { tag: 'err', val: ... }
		if (result && typeof result === 'object' && 'tag' in result) {
			if (result.tag === 'err') {
				throw new Error(result.val);
			}
			const sol = result.val;
			paths = sol.paths; // Already structured: list<path> -> { steps: list<coordinate> }[]
			cost = Number(sol.cost);
			
			// If Component doesn't return Makespan by default but SumOfCosts, we might need to recalculate
			// But the WIT defines `cost` in the record. We trust the solver or recalculate.
			// Let's recalculate Makespan for consistency if we want
			const calculatedMakespan = paths.reduce((max, path) => {
				const pathCost = Math.max(0, path.steps.length - 1);
				return Math.max(max, pathCost);
			}, 0);
			cost = calculatedMakespan;
			
			// Stats?
			const stats = wasmModule.get_stats ? wasmModule.get_stats() : null;
			if (stats) {
				nodesExpanded = Number(stats['nodes-expanded']);
			}
		} else {
			// Direct return (if no error variant used, but we used result<...>)
			// Fallback if binding differs
			paths = result.paths;
			cost = Number(result.cost);
		}
	}

	const endTime = performance.now();

	sendResponse(id, {
		paths,
		cost,
		nodesExpanded,
		timeUs: Math.round((endTime - startTime) * 1000)
	});
}

function unflattenCoords(flat: Uint32Array): Coordinate[] {
	const result: Coordinate[] = [];
	for (let i = 0; i < flat.length; i += 2) {
		result.push({ x: flat[i], y: flat[i+1] });
	}
	return result;
}

/**
 * Parse flattened paths from Uint32Array to structured format.
 * Format: [path1_len, x1, y1, x2, y2, ..., path2_len, ...]
 */
function parseFlattenedPaths(data: Uint32Array): { steps: Coordinate[] }[] {
	const paths: { steps: Coordinate[] }[] = [];
	let i = 0;
	
	while (i < data.length) {
		const pathLen = data[i++];
		const steps: Coordinate[] = [];
		
		for (let j = 0; j < pathLen; j++) {
			const x = data[i++];
			const y = data[i++];
			steps.push({ x, y });
		}
		
		paths.push({ steps });
	}
	
	return paths;
}

/**
 * Get solver info.
 */
function handleInfo(id: number): void {
	if (!wasmModule) {
		throw new Error('WASM module not initialized');
	}

	let info = 'Unknown Solver';
	if (wasmModule.solver_info) {
		info = wasmModule.solver_info();
	} else if (wasmModule.info) {
		info = wasmModule.info();
	}

	sendResponse(id, { info });
}

/**
 * Send a success response.
 */
function sendResponse(id: number, data: WorkerResponse['data']): void {
	const response: WorkerResponse = { id, success: true, data };
	self.postMessage(response);
}

/**
 * Send an error response.
 */
function sendError(id: number, error: string): void {
	const response: WorkerResponse = { id, success: false, error };
	self.postMessage(response);
}
