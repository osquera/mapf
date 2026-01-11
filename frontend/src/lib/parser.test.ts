// Tests for MovingAI map parser

import { describe, it, expect } from 'vitest';
import { parseMap, parseScenario, mapToBytes } from './parser';

describe('parseMap', () => {
	const EMPTY_8X8 = `type octile
height 8
width 8
map
........
........
........
........
........
........
........
........
`;

	const MAZE_SNIPPET = `type octile
height 4
width 6
map
@@@@@@
@....@
@.@@.@
@@@@@@
`;

	it('parses an empty 8x8 map', () => {
		const map = parseMap(EMPTY_8X8);
		expect(map.width).toBe(8);
		expect(map.height).toBe(8);
		expect(map.tiles.length).toBe(64);
		// All passable
		expect(map.tiles.every((t) => t === 1)).toBe(true);
	});

	it('parses a maze with walls', () => {
		const map = parseMap(MAZE_SNIPPET);
		expect(map.width).toBe(6);
		expect(map.height).toBe(4);
		// Corners are blocked
		expect(map.tiles[0]).toBe(0); // (0,0)
		expect(map.tiles[5]).toBe(0); // (5,0)
		// Interior passable
		expect(map.tiles[1 * 6 + 1]).toBe(1); // (1,1)
		// Interior wall
		expect(map.tiles[2 * 6 + 2]).toBe(0); // (2,2)
	});

	it('throws on missing header', () => {
		expect(() => parseMap('map\n....')).toThrow(/width|height/i);
	});

	it('throws on dimension mismatch', () => {
		const bad = `type octile
height 2
width 4
map
....
`;
		expect(() => parseMap(bad)).toThrow(/row/i);
	});
});

describe('parseScenario', () => {
	const SIMPLE_SCEN = `version 1
0	empty-8-8.map	8	8	0	0	1	0	1.00000000
0	empty-8-8.map	8	8	5	3	5	6	3.00000000
`;

	it('parses a simple scenario', () => {
		const scen = parseScenario(SIMPLE_SCEN);
		expect(scen.version).toBe(1);
		expect(scen.entries.length).toBe(2);

		const e0 = scen.entries[0];
		expect(e0.bucket).toBe(0);
		expect(e0.mapName).toBe('empty-8-8.map');
		expect(e0.startX).toBe(0);
		expect(e0.startY).toBe(0);
		expect(e0.goalX).toBe(1);
		expect(e0.goalY).toBe(0);
		expect(e0.optimalLength).toBeCloseTo(1.0);
	});

	it('throws on missing version', () => {
		expect(() => parseScenario('0\tempty.map\t8\t8\t0\t0\t1\t0\t1.0')).toThrow(/version/i);
	});

	it('handles empty scenario after version', () => {
		const scen = parseScenario('version 1\n');
		expect(scen.entries.length).toBe(0);
	});
});

describe('mapToBytes', () => {
	it('converts map to byte array', () => {
		const map = parseMap(`type octile
height 2
width 3
map
.@.
@.@
`);
		const bytes = mapToBytes(map);
		expect(bytes).toEqual(new Uint8Array([1, 0, 1, 0, 1, 0]));
	});
});
