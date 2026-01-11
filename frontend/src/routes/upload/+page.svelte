<script lang="ts">
	let file: File | null = $state(null);
	let uploading = $state(false);
	let error: string | null = $state(null);
	let success = $state(false);

	function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		file = input.files?.[0] ?? null;
		error = null;
		success = false;
	}

	async function handleUpload() {
		if (!file) {
			error = 'Please select a .wasm file';
			return;
		}

		if (!file.name.endsWith('.wasm')) {
			error = 'Only .wasm files are accepted';
			return;
		}

		uploading = true;
		error = null;

		try {
			// TODO: Implement actual upload and browser smoke test
			// For now, just simulate
			await new Promise((resolve) => setTimeout(resolve, 1000));
			success = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Upload failed';
		} finally {
			uploading = false;
		}
	}
</script>

<h1>📤 Upload Solver</h1>
<p>Upload your WASM solver to benchmark it against the standard maps.</p>

<div class="upload-area">
	<h2>1. Select your solver</h2>
	<p class="hint">
		Your solver must be compiled to WebAssembly (.wasm) and implement the
		<a href="https://github.com/your-repo/mapf-arena/blob/main/solvers/wit/mapf-solver.wit"
			>solver interface</a
		>.
	</p>

	<div class="file-input">
		<input type="file" accept=".wasm" onchange={handleFileSelect} />
		{#if file}
			<span class="file-name">{file.name} ({(file.size / 1024).toFixed(1)} KB)</span>
		{/if}
	</div>

	<h2>2. Run smoke test</h2>
	<p class="hint">
		We'll run your solver on a small map in your browser to verify it works before server
		verification.
	</p>

	<button onclick={handleUpload} disabled={!file || uploading}>
		{uploading ? 'Testing...' : 'Upload & Test'}
	</button>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	{#if success}
		<div class="success">
			✅ Smoke test passed! Your solver has been queued for server verification.
		</div>
	{/if}
</div>

<div class="requirements">
	<h2>Solver Requirements</h2>
	<ul>
		<li>Compiled to <code>wasm32-wasi</code> or <code>wasm32-unknown-unknown</code></li>
		<li>
			Exports the <code>solve</code> function per the
			<a href="/docs/wit">WIT interface</a>
		</li>
		<li>No external dependencies (filesystem, network)</li>
		<li>Deterministic output for the same input</li>
		<li>Maximum binary size: 10 MB</li>
	</ul>
</div>

<style>
	.upload-area {
		background: #1a1a2e;
		border-radius: 8px;
		padding: 1.5rem;
		margin: 1.5rem 0;
	}

	.upload-area h2 {
		margin-top: 0;
		font-size: 1.1rem;
		color: #4fc3f7;
	}

	.hint {
		color: #888;
		font-size: 0.9rem;
		margin-bottom: 1rem;
	}

	.file-input {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	input[type='file'] {
		background: #2a2a4a;
		border: 1px solid #3a3a5a;
		border-radius: 4px;
		padding: 0.5rem;
		color: #eee;
	}

	.file-name {
		color: #4fc3f7;
	}

	button {
		background: #4fc3f7;
		color: #000;
		border: none;
		border-radius: 4px;
		padding: 0.75rem 1.5rem;
		cursor: pointer;
		font-weight: 500;
		font-size: 1rem;
	}

	button:hover:not(:disabled) {
		background: #81d4fa;
	}

	button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.error {
		background: #4a1a1a;
		border: 1px solid #f44336;
		border-radius: 4px;
		padding: 0.75rem;
		color: #f44336;
		margin-top: 1rem;
	}

	.success {
		background: #1a4a1a;
		border: 1px solid #4caf50;
		border-radius: 4px;
		padding: 0.75rem;
		color: #4caf50;
		margin-top: 1rem;
	}

	.requirements {
		background: #1a1a2e;
		border-radius: 8px;
		padding: 1.5rem;
	}

	.requirements h2 {
		margin-top: 0;
		color: #4fc3f7;
	}

	.requirements ul {
		color: #aaa;
		padding-left: 1.5rem;
	}

	.requirements li {
		margin: 0.5rem 0;
	}

	code {
		background: #2a2a4a;
		padding: 0.1rem 0.3rem;
		border-radius: 3px;
		font-size: 0.9em;
	}
</style>
