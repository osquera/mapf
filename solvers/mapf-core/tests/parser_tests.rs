//! Tests for MovingAI map and scenario parsing.

use mapf_core::{GridMap, MapError, Scenario, ScenarioEntry, ScenarioError, Tile};
use pretty_assertions::assert_eq;

// ─────────────────────────────────────────────────────────────────────────────
// GridMap tests
// ─────────────────────────────────────────────────────────────────────────────

const EMPTY_8X8_MAP: &str = r#"type octile
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
"#;

const MAZE_SNIPPET: &str = r#"type octile
height 4
width 6
map
@@@@@@
@....@
@.@@.@
@@@@@@
"#;

#[test]
fn parse_empty_8x8_map() {
    let map = GridMap::parse(EMPTY_8X8_MAP).expect("parse failed");
    assert_eq!(map.width(), 8);
    assert_eq!(map.height(), 8);
    // All cells passable
    for y in 0..8 {
        for x in 0..8 {
            assert_eq!(map.get(x, y), Some(Tile::Passable), "({x},{y}) should be passable");
        }
    }
}

#[test]
fn parse_maze_with_walls() {
    let map = GridMap::parse(MAZE_SNIPPET).expect("parse failed");
    assert_eq!(map.width(), 6);
    assert_eq!(map.height(), 4);
    // Corners are walls
    assert_eq!(map.get(0, 0), Some(Tile::Blocked));
    assert_eq!(map.get(5, 0), Some(Tile::Blocked));
    // Interior passable
    assert_eq!(map.get(1, 1), Some(Tile::Passable));
    assert_eq!(map.get(2, 1), Some(Tile::Passable));
    // Interior wall
    assert_eq!(map.get(2, 2), Some(Tile::Blocked));
}

#[test]
fn map_get_out_of_bounds() {
    let map = GridMap::parse(MAZE_SNIPPET).unwrap();
    assert_eq!(map.get(100, 100), None);
}

#[test]
fn map_is_passable() {
    let map = GridMap::parse(MAZE_SNIPPET).unwrap();
    assert!(map.is_passable(1, 1));
    assert!(!map.is_passable(0, 0));
    assert!(!map.is_passable(100, 100)); // out of bounds = not passable
}

#[test]
fn map_parse_missing_header() {
    let bad = "map\n....";
    let err = GridMap::parse(bad).unwrap_err();
    assert!(matches!(err, MapError::MissingHeader(_)));
}

#[test]
fn map_parse_dimension_mismatch() {
    let bad = r#"type octile
height 2
width 4
map
....
"#;
    let err = GridMap::parse(bad).unwrap_err();
    assert!(matches!(err, MapError::DimensionMismatch { .. }));
}

#[test]
fn map_to_bytes_roundtrip() {
    let map = GridMap::parse(MAZE_SNIPPET).unwrap();
    let bytes = map.to_bytes();
    assert_eq!(bytes.len(), 6 * 4); // width * height
    // Verify first row is all blocked (0)
    assert!(bytes[0..6].iter().all(|&b| b == 0));
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario tests
// ─────────────────────────────────────────────────────────────────────────────

const SIMPLE_SCEN: &str = r#"version 1
0	empty-8-8.map	8	8	0	0	1	0	1.00000000
0	empty-8-8.map	8	8	5	3	5	6	3.00000000
"#;

#[test]
fn parse_simple_scenario() {
    let scen = Scenario::parse(SIMPLE_SCEN).expect("parse failed");
    assert_eq!(scen.version(), 1);
    assert_eq!(scen.entries().len(), 2);

    let e0 = &scen.entries()[0];
    assert_eq!(e0.bucket, 0);
    assert_eq!(e0.map_name, "empty-8-8.map");
    assert_eq!(e0.map_width, 8);
    assert_eq!(e0.map_height, 8);
    assert_eq!(e0.start_x, 0);
    assert_eq!(e0.start_y, 0);
    assert_eq!(e0.goal_x, 1);
    assert_eq!(e0.goal_y, 0);
    assert!((e0.optimal_length - 1.0).abs() < 1e-6);

    let e1 = &scen.entries()[1];
    assert_eq!(e1.start_x, 5);
    assert_eq!(e1.goal_y, 6);
}

#[test]
fn scenario_parse_missing_version() {
    let bad = "0\tempty.map\t8\t8\t0\t0\t1\t0\t1.0";
    let err = Scenario::parse(bad).unwrap_err();
    assert!(matches!(err, ScenarioError::MissingVersion));
}

#[test]
fn scenario_parse_malformed_entry() {
    let bad = "version 1\nonly three columns";
    let err = Scenario::parse(bad).unwrap_err();
    assert!(matches!(err, ScenarioError::MalformedEntry { .. }));
}

#[test]
fn scenario_empty_after_version() {
    let input = "version 1\n";
    let scen = Scenario::parse(input).unwrap();
    assert_eq!(scen.entries().len(), 0);
}

#[test]
fn scenario_agents_extraction() {
    let scen = Scenario::parse(SIMPLE_SCEN).unwrap();
    let (starts, goals) = scen.agents();
    assert_eq!(starts.len(), 2);
    assert_eq!(goals.len(), 2);
    assert_eq!(starts[0], (0, 0));
    assert_eq!(goals[0], (1, 0));
}
