// MovingAI map and scenario parser (TypeScript implementation)

import type { GridMap, Scenario, ScenarioEntry } from './types';

/**
 * Parse a MovingAI .map file content.
 *
 * Format:
 * ```
 * type octile
 * height N
 * width M
 * map
 * <N rows of M chars>
 * ```
 */
export function parseMap(input: string): GridMap {
	const lines = input.split(/\r?\n/);
	let width: number | null = null;
	let height: number | null = null;
	let mapStartIndex = -1;

	// Parse header
	for (let i = 0; i < lines.length; i++) {
		const line = lines[i].trim();
		if (line.toLowerCase() === 'map') {
			mapStartIndex = i + 1;
			break;
		}
		if (line.startsWith('height ')) {
			height = parseInt(line.slice(7), 10);
		} else if (line.startsWith('width ')) {
			width = parseInt(line.slice(6), 10);
		}
	}

	if (width === null) throw new Error('Missing width in map header');
	if (height === null) throw new Error('Missing height in map header');
	if (mapStartIndex < 0) throw new Error('Missing "map" marker');

	// Parse grid
	const tiles = new Uint8Array(width * height);
	let rowCount = 0;

	for (let i = mapStartIndex; i < lines.length && rowCount < height; i++) {
		const row = lines[i];
		if (row.length < width) {
			throw new Error(`Row ${rowCount} too short: expected ${width}, got ${row.length}`);
		}

		for (let x = 0; x < width; x++) {
			const ch = row[x];
			// . G S are passable; everything else is blocked
			tiles[rowCount * width + x] = ch === '.' || ch === 'G' || ch === 'S' ? 1 : 0;
		}
		rowCount++;
	}

	if (rowCount < height) {
		throw new Error(`Missing rows: expected ${height}, got ${rowCount}`);
	}

	return { width, height, tiles };
}

/**
 * Parse a MovingAI .scen file content.
 *
 * Format:
 * ```
 * version N
 * bucket\tmap\twidth\theight\tstart_x\tstart_y\tgoal_x\tgoal_y\toptimal
 * ...
 * ```
 */
export function parseScenario(input: string): Scenario {
	const lines = input.split(/\r?\n/);
	let version: number | null = null;
	const entries: ScenarioEntry[] = [];

	for (const line of lines) {
		const trimmed = line.trim();
		if (!trimmed) continue;

		if (version === null) {
			if (!trimmed.startsWith('version ')) {
				throw new Error('Missing version header in scenario');
			}
			version = parseInt(trimmed.slice(8), 10);
			continue;
		}

		const parts = trimmed.split('\t');
		if (parts.length < 9) continue; // skip malformed lines

		entries.push({
			bucket: parseInt(parts[0], 10),
			mapName: parts[1],
			mapWidth: parseInt(parts[2], 10),
			mapHeight: parseInt(parts[3], 10),
			startX: parseInt(parts[4], 10),
			startY: parseInt(parts[5], 10),
			goalX: parseInt(parts[6], 10),
			goalY: parseInt(parts[7], 10),
			optimalLength: parseFloat(parts[8])
		});
	}

	if (version === null) {
		throw new Error('Missing version header in scenario');
	}

	return { version, entries };
}

/**
 * Convert a GridMap to a flat byte array for solver input.
 */
export function mapToBytes(map: GridMap): Uint8Array {
	return map.tiles;
}

/**
 * Check if a coordinate is passable in the map.
 */
export function isPassable(map: GridMap, x: number, y: number): boolean {
	if (x < 0 || x >= map.width || y < 0 || y >= map.height) return false;
	return map.tiles[y * map.width + x] === 1;
}
