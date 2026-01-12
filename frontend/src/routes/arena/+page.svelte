<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import MapViewer from '$lib/components/MapViewer.svelte';
	import { parseMap, parseScenario } from '$lib/parser';
	import { SolverRunner } from '$lib/solver';
	import { backendClient, type VerifyResponse } from '$lib/api';
	import type { GridMap, Scenario, Path, Coordinate, SolverResult } from '$lib/types';

	let map: GridMap | null = $state(null);
	let scenario: Scenario | null = $state(null);
	let paths: Path[] = $state([]);
	let agents: { start: Coordinate; goal: Coordinate }[] = $state([]);
	let error: string | null = $state(null);

	// Solver state
	let runner: SolverRunner | null = $state(null);
	let solverReady: boolean = $state(false);
	let solving: boolean = $state(false);
	let solverResult: SolverResult | null = $state(null);

	// Backend verification state
	let backendAvailable: boolean = $state(false);
	let verifyingOnBackend: boolean = $state(false);
	let backendResult: VerifyResponse | null = $state(null);
	let currentWasmBytes: Uint8Array | null = $state(null);

	// Map selection state
	let sourceMode: 'upload' | 'select' = $state('select');
	let mapIndex: { maps: string[]; scenarios: Record<string, string[]> } | null = $state(null);
	let selectedMapName: string = $state('');
	let selectedScenarioName: string = $state('');
	let availableScenarios: string[] = $state([]);

	// Solver options
	let selectedAlgorithm = $state('A*');
	let selectedHeuristic = $state('Manhattan');
	const algorithms = ['A*', 'CBS (Future)', 'PBS (Future)'];
	const heuristics = ['Manhattan', 'Euclidean', 'Chebyshev', 'Octile'];

	// Sample map for demo
	const SAMPLE_MAP = `type octile
height 16
width 16
map
................
................
..@@@@@@@@@@@@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@..........@..
..@@@@@@@@@@@@..
................
................
`;

	function loadSampleMap() {
		try {
			map = parseMap(SAMPLE_MAP);
			// Add sample agents for multi-agent demo
			agents = [
				{ start: { x: 0, y: 0 }, goal: { x: 15, y: 15 } },
				{ start: { x: 15, y: 0 }, goal: { x: 0, y: 15 } }
			];
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load map';
		}
	}

	function handleMapUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		const reader = new FileReader();
		reader.onload = (e) => {
			try {
				const content = e.target?.result as string;
				map = parseMap(content);
				agents = [];
				paths = [];
				error = null;
			} catch (err) {
				error = err instanceof Error ? err.message : 'Failed to parse map';
			}
		};
		reader.readAsText(file);
	}

	function handleScenarioUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		const reader = new FileReader();
		reader.onload = (e) => {
			try {
				const content = e.target?.result as string;
				scenario = parseScenario(content);
				// Load first few agents from scenario
				agents = scenario.entries.slice(0, 5).map((entry) => ({
					start: { x: entry.startX, y: entry.startY },
					goal: { x: entry.goalX, y: entry.goalY }
				}));
				error = null;
			} catch (err) {
				error = err instanceof Error ? err.message : 'Failed to parse scenario';
			}
		};
		reader.readAsText(file);
	}

	// Load sample on mount
	onMount(async () => {
		loadSampleMap();
		initSolver();
		
		// Check backend availability
		backendAvailable = await backendClient.healthCheck();
		console.log('Backend available:', backendAvailable);
		
		try {
			const res = await fetch('/maps/index.json');
			if (res.ok) {
				mapIndex = await res.json();
			}
		} catch (e) {
			console.warn('Failed to load map index', e);
		}
	});

	async function handleMapSelect() {
		if (!selectedMapName) return;
		try {
			const res = await fetch(`/maps/${selectedMapName}`);
			if (!res.ok) throw new Error('Failed to fetch map');
			const content = await res.text();
			map = parseMap(content);
			agents = [];
			paths = [];
			error = null;
			
			// Update available scenarios
			if (mapIndex && mapIndex.scenarios[selectedMapName]) {
				availableScenarios = mapIndex.scenarios[selectedMapName];
				selectedScenarioName = '';
			} else {
				availableScenarios = [];
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load map';
		}
	}

	async function handleScenarioSelect() {
		if (!selectedScenarioName) return;
		try {
			const res = await fetch(`/maps/${selectedScenarioName}`);
			if (!res.ok) throw new Error('Failed to fetch scenario');
			const content = await res.text();
			scenario = parseScenario(content);
			// Load first few agents from scenario
			agents = scenario.entries.slice(0, 5).map((entry) => ({
				start: { x: entry.startX, y: entry.startY },
				goal: { x: entry.goalX, y: entry.goalY }
			}));
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load scenario';
		}
	}

	// Cleanup on unmount
	onDestroy(() => {
		runner?.terminate();
	});

	let solverMode: 'builtin' | 'custom' = $state('builtin');
	let solverName = $state('Built-in A*');

	async function initSolver() {
		try {
			if (solverMode === 'builtin') {
				runner = new SolverRunner();
				// Use the built-in A* solver WASM
				const wasmUrl = '/wasm/mapf_astar_bg.wasm';
				const jsUrl = '/wasm/mapf_astar.js';
				const name = await runner.init(wasmUrl, jsUrl);
				solverName = name;
				solverReady = true;
				error = null;
			}
		} catch (e) {
			error = `Failed to initialize solver: ${e instanceof Error ? e.message : String(e)}`;
			solverReady = false;
		}
	}

	async function handleSolverUpload(event: Event) {
		console.log('Upload started');
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		solverReady = false;
		error = null;

		try {
			console.log('Reading file...');
			const buffer = await file.arrayBuffer();
			console.log(`Buffer read: ${buffer.byteLength} bytes`);
			
			// Save WASM bytes for backend verification
			currentWasmBytes = new Uint8Array(buffer);
			
			// Custom WASM can only run on backend (Cloudflare doesn't support JCO)
			if (!backendAvailable) {
				throw new Error(
					'Backend server is not available. Custom WASM solvers require backend verification. ' +
					'Please start the backend server or use the built-in solver.'
				);
			}
			
			// Set custom mode without browser initialization
			solverMode = 'custom';
			solverName = file.name.replace('.wasm', '');
			solverReady = true;
			
			console.log(`Custom solver "${solverName}" ready for backend verification`);
		} catch (e) {
			console.error('Upload failed:', e);
			error = `Failed to load custom solver: ${e instanceof Error ? e.message : String(e)}`;
			currentWasmBytes = null;
		}
	}

	async function runSolver() {
		if (!map || agents.length === 0) return;

		solving = true;
		solverResult = null;
		backendResult = null;
		paths = [];
		error = null;

		const starts = agents.map((a) => a.start);
		const goals = agents.map((a) => a.goal);

		try {
			// Custom WASM: Skip browser, run on backend only
			if (solverMode === 'custom' && currentWasmBytes) {
				if (!backendAvailable) {
					throw new Error('Backend server required for custom WASM solvers');
				}
				
				console.log('Running custom solver on backend only...');
				await verifyOnBackend(starts, goals);
				
				// Display backend results as paths
				if (backendResult?.valid && backendResult.solution?.paths) {
					paths = backendResult.solution.paths;
				}
			}
			// Built-in solver: Run browser smoke test first
			else if (runner) {
				const result = await runner.solve(map, starts, goals, {
					algorithm: selectedAlgorithm,
					heuristic: selectedHeuristic
				});
				solverResult = result;

				if (result.status === 'success' && result.solution?.paths) {
					paths = result.solution.paths;
					
					// If backend is available, also verify on server
					if (backendAvailable && !verifyingOnBackend) {
						verifyOnBackend(starts, goals);
					}
				} else if (result.error) {
					error = result.error;
				}
			} else {
				throw new Error('No solver initialized');
			}
		} catch (e) {
			error = `Solver error: ${e instanceof Error ? e.message : String(e)}`;
		} finally {
			solving = false;
		}
	}

	async function verifyOnBackend(starts: Coordinate[], goals: Coordinate[]) {
		if (!currentWasmBytes || !map) return;
		
		verifyingOnBackend = true;
		backendResult = null;

		try {
			console.log('Verifying on backend...');
			const result = await backendClient.verify({
				wasmBytes: currentWasmBytes,
				map: {
					width: map.width,
					height: map.height,
					tiles: Array.from(map.tiles)
				},
				starts,
				goals
			});
			
			backendResult = result;
			console.log('Backend verification result:', result);
		} catch (e) {
			console.error('Backend verification failed:', e);
			// Don't set error state - backend verification is optional
		} finally {
			verifyingOnBackend = false;
		}
	}
</script>

<h1>🎮 Arena</h1>
<p>Load a map and scenario, then run solvers to find paths.</p>

<div class="controls">
	<div class="mode-toggle">
		<button 
			class:active={sourceMode === 'select'} 
			onclick={() => sourceMode = 'select'}
		>
			Select Preset
		</button>
		<button 
			class:active={sourceMode === 'upload'} 
			onclick={() => sourceMode = 'upload'}
		>
			Upload File
		</button>
	</div>

	{#if sourceMode === 'upload'}
		<div class="control-group">
			<label>
				Map (.map):
				<input type="file" accept=".map" onchange={handleMapUpload} />
			</label>
		</div>

		<div class="control-group">
			<label>
				Scenario (.scen):
				<input type="file" accept=".scen" onchange={handleScenarioUpload} />
			</label>
		</div>
	{:else}
		<div class="control-group">
			<label>
				Map:
				<select bind:value={selectedMapName} onchange={handleMapSelect} disabled={!mapIndex}>
					<option value="">Select a map...</option>
					{#if mapIndex}
						{#each mapIndex.maps as m}
							<option value={m}>{m}</option>
						{/each}
					{/if}
				</select>
			</label>
		</div>

		<div class="control-group">
			<label>
				Scenario:
				<select bind:value={selectedScenarioName} onchange={handleScenarioSelect} disabled={!selectedMapName || availableScenarios.length === 0}>
					<option value="">Select a scenario...</option>
					{#each availableScenarios as s}
						<option value={s}>{s.split('/').pop()}</option>
					{/each}
				</select>
			</label>
		</div>
	{/if}

	<button onclick={loadSampleMap}>Load Sample Map</button>

	<div class="control-group">
		<label>
			Algorithm:
			<select bind:value={selectedAlgorithm}>
				{#each algorithms as algo}
					<option value={algo}>{algo}</option>
				{/each}
			</select>
		</label>
	</div>

	<div class="control-group">
		<label>
			Heuristic:
			<select bind:value={selectedHeuristic}>
				{#each heuristics as h}
					<option value={h}>{h}</option>
				{/each}
			</select>
		</label>
	</div>

	<button onclick={runSolver} disabled={!solverReady || !map || agents.length === 0 || solving}>
		{#if solving}
			⏳ Solving...
		{:else if !solverReady}
			⏳ Loading Solver...
		{:else}
			▶️ Run {solverName}
		{/if}
	</button>
</div>

<div class="solver-upload">
	<h3>Solver Engine</h3>
	<div class="mode-toggle">
		<button 
			class:active={solverMode === 'builtin'} 
			onclick={() => { solverMode = 'builtin'; initSolver(); }}
		>
			Built-in
		</button>
		<label class="upload-btn" class:active={solverMode === 'custom'}>
			Upload .wasm
			<input type="file" accept=".wasm" onchange={handleSolverUpload} style="display: none;" />
		</label>
	</div>
	{#if solverMode === 'custom' && solverReady}
		<p class="solver-status">✅ Loaded: {solverName}</p>
	{/if}
	<p class="download-link">
		<a href="/test-solver.wasm" download>Download Sample Solver WASM</a>
	</p>
</div>

{#if error}
	<div class="error">{error}</div>
{/if}

{#if map}
	<div class="arena">
		<div class="map-container">
			<MapViewer width={map.width} height={map.height} tiles={map.tiles} {paths} {agents} />
		</div>

		<div class="info">
			<h3>Map Info</h3>
			<p>Size: {map.width} × {map.height}</p>
			<p>Agents: {agents.length}</p>

			{#if scenario}
				<h3>Scenario</h3>
				<p>Version: {scenario.version}</p>
				<p>Entries: {scenario.entries.length}</p>
			{/if}

			{#if solverResult}
				<h3>Browser Smoke Test</h3>
				<p class:success={solverResult.status === 'success'} class:failure={solverResult.status !== 'success'}>
					{solverResult.status === 'success' ? '✅ Solution found' : solverResult.status === 'timeout' ? '⏱️ Timeout' : '❌ No solution'}
				</p>
				{#if solverResult.stats}
					<p>Time: {(solverResult.stats.timeUs / 1000).toFixed(2)} ms</p>
					<p>Nodes expanded: {solverResult.stats.nodesExpanded.toLocaleString()}</p>
				{/if}
				{#if solverResult.solution?.paths}
					<p>Paths: {solverResult.solution.paths.length}</p>
					<p>Total cost: {solverResult.solution.cost}</p>
					{#each solverResult.solution.paths as path, i}
						<p class="path-info">Agent {i + 1}: {path.steps.length} steps</p>
					{/each}
				{/if}
			{/if}
			
			{#if backendAvailable && currentWasmBytes}
				<h3>Backend Verification</h3>
				{#if verifyingOnBackend}
					<p class="verifying">🔄 Verifying on server...</p>
				{:else if backendResult}
					<p class:success={backendResult.valid} class:failure={!backendResult.valid}>
						{backendResult.valid ? '✅ Server verified' : '❌ Invalid solution'}
					</p>
					{#if backendResult.stats.instruction_count}
						<p><strong>Instructions:</strong> {backendResult.stats.instruction_count.toLocaleString()}</p>
					{/if}
					<p>Server time: {backendResult.stats.execution_time_ms} ms</p>
					{#if backendResult.stats.cost}
						<p>Cost: {backendResult.stats.cost}</p>
					{/if}
					{#if backendResult.stats.makespan}
						<p>Makespan: {backendResult.stats.makespan}</p>
					{/if}
					{#if backendResult.error}
						<p class="error">Error: {backendResult.error}</p>
					{/if}
					{#if backendResult.validation_errors.length > 0}
						<details>
							<summary>Validation errors ({backendResult.validation_errors.length})</summary>
							<ul class="validation-errors">
								{#each backendResult.validation_errors as err}
									<li>{err.details}</li>
								{/each}
							</ul>
						</details>
					{/if}
				{/if}
			{/if}
		</div>
	</div>
{/if}

<style>
	.controls {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		margin-bottom: 1.5rem;
		align-items: end;
	}

	.mode-toggle {
		display: flex;
		gap: 0.5rem;
		margin-right: 1rem;
	}

	.mode-toggle button {
		background: #2a2a4a;
		color: #aaa;
		border: 1px solid #3a3a5a;
	}

	.mode-toggle button.active {
		background: #4fc3f7;
		color: #000;
		border-color: #4fc3f7;
	}

	.control-group {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	label {
		color: #aaa;
		font-size: 0.9rem;
	}

	input[type='file'], select {
		background: #2a2a4a;
		border: 1px solid #3a3a5a;
		border-radius: 4px;
		padding: 0.5rem;
		color: #eee;
	}

	button {
		background: #4fc3f7;
		color: #000;
		border: none;
		border-radius: 4px;
		padding: 0.5rem 1rem;
		cursor: pointer;
		font-weight: 500;
	}

	button:hover {
		background: #81d4fa;
	}

	.error {
		background: #4a1a1a;
		border: 1px solid #f44336;
		border-radius: 4px;
		padding: 0.75rem;
		color: #f44336;
		margin-bottom: 1rem;
	}

	.arena {
		display: flex;
		gap: 2rem;
		flex-wrap: wrap;
	}

	.map-container {
		flex-shrink: 0;
	}

	.info {
		background: #1a1a2e;
		border-radius: 8px;
		padding: 1rem;
		min-width: 200px;
	}

	.info h3 {
		margin-top: 0;
		color: #4fc3f7;
	}

	.info p {
		margin: 0.5rem 0;
		color: #aaa;
	}

	.info p.success {
		color: #4caf50;
	}

	.info p.failure {
		color: #f44336;
	}

	.info p.verifying {
		color: #4fc3f7;
		font-style: italic;
	}

	.info p.error {
		color: #ff6b6b;
		font-size: 0.9rem;
	}

	.path-info {
		font-size: 0.85rem;
		margin-left: 0.5rem;
	}

	.validation-errors {
		margin-top: 0.5rem;
		padding-left: 1.5rem;
		color: #ff6b6b;
		font-size: 0.85rem;
	}

	.validation-errors li {
		margin: 0.25rem 0;
	}

	details {
		margin-top: 0.5rem;
		cursor: pointer;
	}

	summary {
		color: #ff6b6b;
		font-size: 0.9rem;
	}

	.solver-upload {
		background: #1a1a2e;
		padding: 1rem;
		border-radius: 8px;
		margin-bottom: 1.5rem;
		border: 1px solid #3a3a5a;
	}
	
	.solver-upload h3 {
		margin-top: 0;
		font-size: 1rem;
		color: #4fc3f7;
		margin-bottom: 0.5rem;
	}

	.upload-btn {
		background: #2a2a4a;
		color: #aaa;
		border: 1px solid #3a3a5a;
		padding: 0.5rem 1rem;
		border-radius: 4px;
		cursor: pointer;
		font-weight: 500;
		display: inline-block;
	}

	.upload-btn.active {
		background: #4fc3f7;
		color: #000;
		border-color: #4fc3f7;
	}

	.solver-status {
		color: #4caf50;
		margin: 0.5rem 0 0 0;
		font-size: 0.9rem;
	}

	.download-link {
		margin-top: 0.5rem;
		font-size: 0.8rem;
	}

	.download-link a {
		color: #aaa;
		text-decoration: underline;
	}

	button:disabled {
		background: #666;
		cursor: not-allowed;
		opacity: 0.6;
	}
</style>
