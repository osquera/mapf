<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();
</script>

<svelte:head>
	<title>MAPF Arena</title>
</svelte:head>

<div class="app">
	<header>
		<nav>
			<a href="/">Home</a>
			<a href="/arena">Arena</a>
			<a href="/leaderboard">Leaderboard</a>
			<a href="/upload">Upload Solver</a>
			<a href="/docs">Docs</a>
		</nav>
	</header>

	<main>
		{@render children()}
	</main>

	<footer>
		<p>MAPF Arena &copy; 2026</p>
	</footer>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	header {
		background: #1a1a2e;
		padding: 1rem;
	}

	nav {
		display: flex;
		gap: 1.5rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	nav a {
		color: #eee;
		text-decoration: none;
		font-weight: 500;
	}

	nav a:hover {
		color: #4fc3f7;
	}

	main {
		flex: 1;
		padding: 2rem;
		max-width: 1200px;
		margin: 0 auto;
		width: 100%;
	}

	footer {
		background: #1a1a2e;
		color: #888;
		padding: 1rem;
		text-align: center;
	}

	:global(body) {
		margin: 0;
		font-family: system-ui, -apple-system, sans-serif;
		background: #0f0f1a;
		color: #eee;
	}

	:global(a) {
		color: #4fc3f7;
	}
</style>
