<script lang="ts">
	import MapViewer from '$lib/components/MapViewer.svelte';
</script>

<section class="hero">
	<h1>🗺️ MAPF Arena</h1>
	<p>Multi-Agent Path Finding benchmarks and leaderboard</p>
</section>

<section class="features">
	<div class="card">
		<h2>🚀 Upload Solvers</h2>
		<p>Upload your WASM solver and benchmark it against standard maps.</p>
		<a href="/upload">Get Started →</a>
	</div>

	<div class="card">
		<h2>🎮 Try the Arena</h2>
		<p>Test solvers in your browser with instant visualization.</p>
		<a href="/arena">Open Arena →</a>
	</div>

	<div class="card">
		<h2>🏆 Leaderboard</h2>
		<p>See how your solver ranks against the competition.</p>
		<a href="/leaderboard">View Rankings →</a>
	</div>
</section>

<section class="demo">
	<h2>Sample Map</h2>
	<MapViewer
		width={16}
		height={16}
		tiles={new Uint8Array(256).fill(1)}
	/>
</section>

<style>
	.hero {
		text-align: center;
		padding: 3rem 0;
	}

	.hero h1 {
		font-size: 3rem;
		margin-bottom: 0.5rem;
	}

	.hero p {
		color: #888;
		font-size: 1.25rem;
	}

	.features {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 1.5rem;
		margin: 2rem 0;
	}

	.card {
		background: #1a1a2e;
		border-radius: 8px;
		padding: 1.5rem;
	}

	.card h2 {
		margin-top: 0;
		font-size: 1.25rem;
	}

	.card p {
		color: #aaa;
		margin-bottom: 1rem;
	}

	.demo {
		margin-top: 2rem;
	}

	.demo h2 {
		margin-bottom: 1rem;
	}
</style>
