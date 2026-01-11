<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { Coordinate, Path } from '$lib/types';

	interface Props {
		width: number;
		height: number;
		tiles: Uint8Array;
		cellSize?: number;
		paths?: Path[];
		agents?: { start: Coordinate; goal: Coordinate }[];
	}

	let { width, height, tiles, cellSize = 16, paths = [], agents = [] }: Props = $props();

	let canvas: HTMLCanvasElement;
	let currentStep = $state(0);
	let isPlaying = $state(false);
	let animationFrame: number;
	let lastTime: number;
	let speed = $state(2); // steps per second

	const maxSteps = $derived(Math.max(0, ...paths.map((p) => p.steps.length - 1)));

	const COLORS = {
		passable: '#2a2a4a',
		blocked: '#1a1a2e',
		grid: '#3a3a5a',
		start: '#4caf50',
		goal: '#f44336',
		path: ['#4fc3f7', '#ffb74d', '#ba68c8', '#81c784', '#ff8a65']
	};

	function togglePlay() {
		isPlaying = !isPlaying;
		if (isPlaying) {
			if (currentStep >= maxSteps) {
				currentStep = 0;
			}
			lastTime = performance.now();
			animationFrame = requestAnimationFrame(animate);
		} else {
			cancelAnimationFrame(animationFrame);
		}
	}

	function animate(time: number) {
		if (!isPlaying) return;
		const dt = (time - lastTime) / 1000;
		lastTime = time;

		currentStep += dt * speed;

		if (currentStep >= maxSteps) {
			currentStep = maxSteps;
			isPlaying = false;
		} else {
			animationFrame = requestAnimationFrame(animate);
		}
	}

	function handleSliderChange(e: Event) {
		const target = e.target as HTMLInputElement;
		currentStep = parseFloat(target.value);
		if (isPlaying) {
			isPlaying = false;
			cancelAnimationFrame(animationFrame);
		}
	}

	function draw() {
		const ctx = canvas?.getContext('2d');
		if (!ctx) return;

		const canvasWidth = width * cellSize;
		const canvasHeight = height * cellSize;

		// Clear
		ctx.fillStyle = COLORS.blocked;
		ctx.fillRect(0, 0, canvasWidth, canvasHeight);

		// Draw tiles
		for (let y = 0; y < height; y++) {
			for (let x = 0; x < width; x++) {
				const idx = y * width + x;
				const isPassable = tiles[idx] === 1;

				ctx.fillStyle = isPassable ? COLORS.passable : COLORS.blocked;
				ctx.fillRect(x * cellSize, y * cellSize, cellSize, cellSize);

				// Grid lines
				ctx.strokeStyle = COLORS.grid;
				ctx.lineWidth = 1;
				ctx.strokeRect(x * cellSize, y * cellSize, cellSize, cellSize);
			}
		}

		// Draw goals
		agents.forEach((agent, i) => {
			// Goal
			ctx.fillStyle = COLORS.goal;
			ctx.beginPath();
			ctx.rect(
				agent.goal.x * cellSize + cellSize / 4,
				agent.goal.y * cellSize + cellSize / 4,
				cellSize / 2,
				cellSize / 2
			);
			ctx.fill();
            
            // Start (only if no paths, otherwise the agent is drawn at currentStep)
            if (paths.length === 0) {
                ctx.fillStyle = COLORS.start;
                ctx.beginPath();
                ctx.arc(
                    agent.start.x * cellSize + cellSize / 2,
                    agent.start.y * cellSize + cellSize / 2,
                    cellSize / 3,
                    0,
                    Math.PI * 2
                );
                ctx.fill();
            }
		});

		// Draw paths (faint lines)
		paths.forEach((path, i) => {
			if (path.steps.length < 2) return;
			ctx.strokeStyle = COLORS.path[i % COLORS.path.length];
			ctx.lineWidth = 2;
			ctx.globalAlpha = 0.3; // Faint
			ctx.beginPath();
			ctx.moveTo(
				path.steps[0].x * cellSize + cellSize / 2,
				path.steps[0].y * cellSize + cellSize / 2
			);
			for (let j = 1; j < path.steps.length; j++) {
				ctx.lineTo(
					path.steps[j].x * cellSize + cellSize / 2,
					path.steps[j].y * cellSize + cellSize / 2
				);
			}
			ctx.stroke();
			ctx.globalAlpha = 1.0;
		});

		// Draw agents at currentStep
		if (paths.length > 0) {
			paths.forEach((path, i) => {
				const color = COLORS.path[i % COLORS.path.length];

				// Interpolate position
				const t = Math.max(0, Math.min(currentStep, path.steps.length - 1));
				const idx = Math.floor(t);
				const frac = t - idx;

				const p1 = path.steps[idx];
				const p2 = path.steps[Math.min(idx + 1, path.steps.length - 1)];

				const x = p1.x + (p2.x - p1.x) * frac;
				const y = p1.y + (p2.y - p1.y) * frac;

				ctx.fillStyle = color;
				ctx.beginPath();
				ctx.arc(
					x * cellSize + cellSize / 2,
					y * cellSize + cellSize / 2,
					cellSize / 2.5,
					0,
					Math.PI * 2
				);
				ctx.fill();

				// Border
				ctx.strokeStyle = '#fff';
				ctx.lineWidth = 1;
				ctx.stroke();
                
                // Agent ID
                ctx.fillStyle = '#000';
                ctx.font = `${cellSize/2}px sans-serif`;
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText((i+1).toString(), x * cellSize + cellSize / 2, y * cellSize + cellSize / 2);
			});
		}
	}

	onMount(() => {
		draw();
	});
    
    onDestroy(() => {
        if (typeof window !== 'undefined') {
            cancelAnimationFrame(animationFrame);
        }
    });

	$effect(() => {
		// Redraw when props or state change
		tiles;
		paths;
		agents;
		currentStep;
		draw();
	});
    
    // Reset currentStep when paths change significantly (e.g. new solution)
    $effect(() => {
        if (paths) {
            // If paths change, we might want to reset, but only if it's a new solve.
            // For now, let's just clamp currentStep.
            if (currentStep > maxSteps) currentStep = 0;
        }
    });
</script>

<div class="map-viewer-container">
	<div class="canvas-wrapper">
		<canvas bind:this={canvas} width={width * cellSize} height={height * cellSize}></canvas>
	</div>
    
    {#if paths.length > 0}
        <div class="controls">
            <button class="play-btn" onclick={togglePlay}>
                {isPlaying ? '⏸️' : '▶️'}
            </button>
            
            <input 
                type="range" 
                min="0" 
                max={maxSteps} 
                step="0.01" 
                value={currentStep} 
                oninput={handleSliderChange}
            />
            
            <span class="step-counter">
                {Math.floor(currentStep)} / {maxSteps}
            </span>
            
            <label class="speed-control">
                Speed:
                <input type="number" bind:value={speed} min="0.1" max="10" step="0.1" />
            </label>
        </div>
    {/if}
</div>

<style>
	.map-viewer-container {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.canvas-wrapper {
		display: inline-block;
		border: 2px solid #3a3a5a;
		border-radius: 4px;
		overflow: hidden;
        background: #1a1a2e;
	}

	canvas {
		display: block;
	}
    
    .controls {
        display: flex;
        align-items: center;
        gap: 1rem;
        background: #2a2a4a;
        padding: 0.5rem;
        border-radius: 4px;
        color: #eee;
    }
    
    .play-btn {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 1.2rem;
        padding: 0;
    }
    
    input[type="range"] {
        flex: 1;
    }
    
    .step-counter {
        font-family: monospace;
        min-width: 4ch;
    }
    
    .speed-control {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.8rem;
    }
    
    .speed-control input {
        width: 3rem;
        background: #1a1a2e;
        border: 1px solid #3a3a5a;
        color: #eee;
        padding: 0.2rem;
        border-radius: 2px;
    }
</style>