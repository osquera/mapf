<script lang="ts">
	// Placeholder leaderboard data
	const leaderboard = [
		{ rank: 1, solver: 'CBS-Optimal', author: 'Example', solved: 850, cost: 12450, time: '2.3s' },
		{ rank: 2, solver: 'PP-A*', author: 'Example', solved: 820, cost: 13200, time: '0.8s' },
		{ rank: 3, solver: 'ECBS', author: 'Example', solved: 800, cost: 12800, time: '1.5s' }
	];
</script>

<h1>🏆 Leaderboard</h1>
<p>Rankings based on the MovingAI benchmark suite.</p>

<table>
	<thead>
		<tr>
			<th>Rank</th>
			<th>Solver</th>
			<th>Author</th>
			<th>Solved</th>
			<th>Total Cost</th>
			<th>Avg Time</th>
		</tr>
	</thead>
	<tbody>
		{#each leaderboard as entry}
			<tr>
				<td class="rank">#{entry.rank}</td>
				<td class="solver">{entry.solver}</td>
				<td>{entry.author}</td>
				<td>{entry.solved}</td>
				<td>{entry.cost}</td>
				<td>{entry.time}</td>
			</tr>
		{/each}
	</tbody>
</table>

<style>
	table {
		width: 100%;
		border-collapse: collapse;
		margin-top: 1.5rem;
	}

	th,
	td {
		padding: 0.75rem 1rem;
		text-align: left;
		border-bottom: 1px solid #3a3a5a;
	}

	th {
		background: #1a1a2e;
		color: #4fc3f7;
		font-weight: 600;
	}

	tr:hover {
		background: #1a1a2e;
	}

	.rank {
		color: #ffb74d;
		font-weight: bold;
	}

	.solver {
		color: #4fc3f7;
	}
</style>
