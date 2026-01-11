//! Tests for A* pathfinding implementation.

use mapf_astar::{astar_single, solve_mapf, Coordinate, Path};

// ─────────────────────────────────────────────────────────────────────────────
// Single-agent A* tests
// ─────────────────────────────────────────────────────────────────────────────

/// 3x3 open grid:
/// ```
/// ...
/// ...
/// ...
/// ```
fn open_3x3() -> Vec<u8> {
    vec![1, 1, 1, 1, 1, 1, 1, 1, 1]
}

/// 3x3 grid with center blocked:
/// ```
/// ...
/// .#.
/// ...
/// ```
fn blocked_center_3x3() -> Vec<u8> {
    vec![1, 1, 1, 1, 0, 1, 1, 1, 1]
}

/// 5x3 corridor with wall:
/// ```
/// .....
/// .###.
/// .....
/// ```
fn corridor_5x3() -> Vec<u8> {
    vec![
        1, 1, 1, 1, 1, // row 0
        1, 0, 0, 0, 1, // row 1
        1, 1, 1, 1, 1, // row 2
    ]
}

#[test]
fn astar_straight_line() {
    // (0,0) to (2,0) in open grid = 2 moves right
    let map = open_3x3();
    let result = astar_single(&map, 3, 3, (0, 0), (2, 0));
    assert!(result.is_some());
    let path = result.unwrap();
    assert_eq!(path.steps.len(), 3); // start + 2 moves
    assert_eq!(path.steps[0], Coordinate { x: 0, y: 0 });
    assert_eq!(path.steps[2], Coordinate { x: 2, y: 0 });
}

#[test]
fn astar_cardinal_only_path() {
    // (0,0) to (2,2) in open grid - must use cardinal moves only
    let map = open_3x3();
    let result = astar_single(&map, 3, 3, (0, 0), (2, 2));
    assert!(result.is_some());
    let path = result.unwrap();
    // Manhattan distance = 4, so path must be exactly 5 steps (start + 4 moves)
    assert_eq!(path.steps.len(), 5, "Path should use Manhattan distance (4 moves)");
    assert_eq!(*path.steps.first().unwrap(), Coordinate { x: 0, y: 0 });
    assert_eq!(*path.steps.last().unwrap(), Coordinate { x: 2, y: 2 });
    // Verify all moves are cardinal (no diagonals)
    assert!(path.is_valid_cardinal(), "Path must use only cardinal moves");
}

#[test]
fn astar_around_obstacle() {
    // (0,1) to (2,1) must go around center
    let map = blocked_center_3x3();
    let result = astar_single(&map, 3, 3, (0, 1), (2, 1));
    assert!(result.is_some());
    let path = result.unwrap();
    // Must not go through (1,1)
    assert!(!path.steps.contains(&Coordinate { x: 1, y: 1 }));
    assert_eq!(*path.steps.first().unwrap(), Coordinate { x: 0, y: 1 });
    assert_eq!(*path.steps.last().unwrap(), Coordinate { x: 2, y: 1 });
}

#[test]
fn astar_same_start_goal() {
    let map = open_3x3();
    let result = astar_single(&map, 3, 3, (1, 1), (1, 1));
    assert!(result.is_some());
    let path = result.unwrap();
    assert_eq!(path.steps.len(), 1);
    assert_eq!(path.steps[0], Coordinate { x: 1, y: 1 });
}

#[test]
fn astar_no_path_blocked_goal() {
    // Goal is blocked
    let mut map = open_3x3();
    map[8] = 0; // block (2,2)
    let result = astar_single(&map, 3, 3, (0, 0), (2, 2));
    assert!(result.is_none());
}

#[test]
fn astar_corridor() {
    // Must navigate around wall
    let map = corridor_5x3();
    let result = astar_single(&map, 5, 3, (0, 1), (4, 1));
    assert!(result.is_some());
    let path = result.unwrap();
    // Path must go through row 0 or row 2
    let goes_through_top = path.steps.iter().any(|c| c.y == 0);
    let goes_through_bottom = path.steps.iter().any(|c| c.y == 2);
    assert!(goes_through_top || goes_through_bottom);
}

#[test]
fn astar_out_of_bounds_start() {
    let map = open_3x3();
    let result = astar_single(&map, 3, 3, (10, 10), (0, 0));
    assert!(result.is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// Path validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn path_cost_calculation() {
    let path = Path {
        steps: vec![
            Coordinate { x: 0, y: 0 },
            Coordinate { x: 1, y: 0 },
            Coordinate { x: 2, y: 0 },
        ],
    };
    // 2 cardinal moves = cost 2
    assert_eq!(path.cost(), 2);
}

#[test]
fn path_cardinal_validation() {
    // Valid cardinal-only path
    let valid_path = Path {
        steps: vec![
            Coordinate { x: 0, y: 0 },
            Coordinate { x: 1, y: 0 }, // East
            Coordinate { x: 1, y: 1 }, // South
            Coordinate { x: 2, y: 1 }, // East
        ],
    };
    assert!(valid_path.is_valid_cardinal());
    assert_eq!(valid_path.cost(), 3);
    
    // Invalid path with diagonal move
    let invalid_path = Path {
        steps: vec![
            Coordinate { x: 0, y: 0 },
            Coordinate { x: 1, y: 1 }, // diagonal - NOT allowed
        ],
    };
    assert!(!invalid_path.is_valid_cardinal());
}

#[test]
fn path_with_noop_validation() {
    // Valid path with NoOp (wait) action
    let path_with_wait = Path {
        steps: vec![
            Coordinate { x: 0, y: 0 },
            Coordinate { x: 1, y: 0 }, // East
            Coordinate { x: 1, y: 0 }, // NoOp (wait)
            Coordinate { x: 2, y: 0 }, // East
        ],
    };
    assert!(path_with_wait.is_valid_cardinal(), "Path with NoOp should be valid");
    assert_eq!(path_with_wait.cost(), 3); // 3 timesteps (move, wait, move)
}

// ─────────────────────────────────────────────────────────────────────────────
// Multi-agent MAPF tests
// ─────────────────────────────────────────────────────────────────────────────

/// 5x5 open grid for multi-agent tests
fn open_5x5() -> Vec<u8> {
    vec![1; 25]
}

/// 10x10 open grid for larger multi-agent tests
fn open_10x10() -> Vec<u8> {
    vec![1; 100]
}

/// 16x16 open grid matching the arena sample
fn open_16x16() -> Vec<u8> {
    vec![1; 256]
}

#[test]
fn multiagent_intersecting_paths_diagonal() {
    // Two agents with paths that MUST intersect at the center
    // Agent 0: (0,0) -> (4,4) - diagonal path via center
    // Agent 1: (4,0) -> (0,4) - diagonal path via center
    // Their optimal paths cross at (2,2)
    let map = open_5x5();
    let agents = vec![
        ((0, 0), (4, 4)),
        ((4, 0), (0, 4)),
    ];
    
    let result = solve_mapf(&map, 5, 5, &agents);
    assert!(result.is_some(), "Should find paths for intersecting agents");
    let paths = result.unwrap();
    assert_eq!(paths.len(), 2);
    
    // Both paths should be valid (cardinal moves + NoOp)
    for (i, path) in paths.iter().enumerate() {
        assert!(path.is_valid_cardinal(), "Agent {} path must use only cardinal moves or NoOp", i);
    }
    
    // Verify correct start/end positions
    assert_eq!(paths[0].steps.first().unwrap(), &Coordinate { x: 0, y: 0 });
    assert_eq!(paths[0].steps.last().unwrap(), &Coordinate { x: 4, y: 4 });
    assert_eq!(paths[1].steps.first().unwrap(), &Coordinate { x: 4, y: 0 });
    assert_eq!(paths[1].steps.last().unwrap(), &Coordinate { x: 0, y: 4 });
    
    // Verify no collisions
    verify_no_collisions(&paths);
}

#[test]
fn multiagent_intersecting_paths_cross() {
    // Two agents crossing perpendicular paths
    // Agent 0: (0,2) -> (4,2) - horizontal through center
    // Agent 1: (2,0) -> (2,4) - vertical through center
    // Their paths MUST cross at (2,2)
    let map = open_5x5();
    let agents = vec![
        ((0, 2), (4, 2)),  // horizontal
        ((2, 0), (2, 4)),  // vertical
    ];
    
    let result = solve_mapf(&map, 5, 5, &agents);
    assert!(result.is_some(), "Should find paths for perpendicular crossing agents");
    let paths = result.unwrap();
    assert_eq!(paths.len(), 2);
    
    // Both paths must pass through or around (2,2)
    for (i, path) in paths.iter().enumerate() {
        assert!(path.is_valid_cardinal(), "Agent {} path must be valid", i);
    }
    
    // Verify no collisions at intersection
    verify_no_collisions(&paths);
}

#[test]
fn multiagent_arena_sample() {
    // This is the exact scenario from the arena sample
    // Agent 0: (0,0) -> (15,15)
    // Agent 1: (15,0) -> (0,15)
    // These paths intersect along the diagonal
    let map = open_16x16();
    let agents = vec![
        ((0, 0), (15, 15)),
        ((15, 0), (0, 15)),
    ];
    
    let result = solve_mapf(&map, 16, 16, &agents);
    assert!(result.is_some(), "Should find paths for arena sample");
    let paths = result.unwrap();
    assert_eq!(paths.len(), 2);
    
    // Verify cardinal moves only (including NoOp)
    for (i, path) in paths.iter().enumerate() {
        assert!(path.is_valid_cardinal(), "Agent {} path must use only cardinal moves or NoOp", i);
    }
    
    // Verify no collisions
    verify_no_collisions(&paths);
}

#[test]
fn multiagent_head_on_corridor() {
    // Two agents moving toward each other on a single row
    // One agent MUST use NoOp (wait) to let the other pass
    // Agent 0: (0,0) -> (4,0)
    // Agent 1: (4,0) -> (0,0)
    let map = open_5x5();
    let agents = vec![
        ((0, 0), (4, 0)),
        ((4, 0), (0, 0)),
    ];
    
    let result = solve_mapf(&map, 5, 5, &agents);
    assert!(result.is_some(), "Should find paths for head-on agents");
    let paths = result.unwrap();
    
    // At least one agent should have waited (NoOp) or taken a detour
    // Both paths must be valid
    for (i, path) in paths.iter().enumerate() {
        assert!(path.is_valid_cardinal(), "Agent {} path must be valid", i);
    }
    
    // Verify no collisions (edge or vertex)
    verify_no_collisions(&paths);
}

#[test]
fn multiagent_swap_positions() {
    // Two agents swapping positions - requires careful coordination
    // Agent 0: (0,0) -> (1,0)
    // Agent 1: (1,0) -> (0,0)
    // This is an edge collision scenario - direct swap is illegal
    let map = open_5x5();
    let agents = vec![
        ((0, 0), (1, 0)),
        ((1, 0), (0, 0)),
    ];
    
    let result = solve_mapf(&map, 5, 5, &agents);
    assert!(result.is_some(), "Should find paths for swapping agents");
    let paths = result.unwrap();
    
    // Verify paths are valid
    for (i, path) in paths.iter().enumerate() {
        assert!(path.is_valid_cardinal(), "Agent {} path must be valid", i);
    }
    
    // Verify no edge collisions (no swapping at same timestep)
    verify_no_collisions(&paths);
}

#[test]
fn multiagent_four_agents_center_cross() {
    // Four agents all meeting at the center - stress test for conflict resolution
    // All paths intersect at (4,4) in a 9x9 grid
    let map = vec![1u8; 81]; // 9x9 grid
    let agents = vec![
        ((0, 4), (8, 4)),  // left to right through center
        ((8, 4), (0, 4)),  // right to left through center
        ((4, 0), (4, 8)),  // top to bottom through center
        ((4, 8), (4, 0)),  // bottom to top through center
    ];
    
    let result = solve_mapf(&map, 9, 9, &agents);
    assert!(result.is_some(), "Should find paths for four agents crossing at center");
    let paths = result.unwrap();
    assert_eq!(paths.len(), 4);
    
    // All paths must be valid
    for (i, path) in paths.iter().enumerate() {
        assert!(path.is_valid_cardinal(), "Agent {} path must be valid", i);
    }
    
    // Verify no collisions
    verify_no_collisions(&paths);
}

#[test]
fn multiagent_three_agents() {
    // Three agents moving to different corners (non-overlapping goals)
    let map = open_10x10();
    let agents = vec![
        ((0, 0), (9, 0)),  // top-left to top-right
        ((0, 9), (9, 9)),  // bottom-left to bottom-right
        ((5, 0), (5, 9)),  // top-middle to bottom-middle
    ];
    
    let result = solve_mapf(&map, 10, 10, &agents);
    assert!(result.is_some(), "Should find paths for three agents");
    let paths = result.unwrap();
    assert_eq!(paths.len(), 3);
    
    verify_no_collisions(&paths);
}

/// Helper function to verify no collisions between paths
fn verify_no_collisions(paths: &[Path]) {
    let max_len = paths.iter().map(|p| p.steps.len()).max().unwrap_or(0);
    
    for t in 0..max_len {
        // Get positions of all agents at timestep t (or their final position)
        let positions: Vec<_> = paths.iter().map(|p| {
            p.steps.get(t).or(p.steps.last()).unwrap()
        }).collect();
        
        // Check for vertex collisions
        for i in 0..positions.len() {
            for j in (i+1)..positions.len() {
                assert_ne!(
                    positions[i], positions[j],
                    "Vertex collision between agents {} and {} at timestep {}: both at ({}, {})",
                    i, j, t, positions[i].x, positions[i].y
                );
            }
        }
        
        // Check for edge collisions (agents swapping positions)
        if t > 0 {
            let prev_positions: Vec<_> = paths.iter().map(|p| {
                p.steps.get(t - 1).or(p.steps.last()).unwrap()
            }).collect();
            
            for i in 0..positions.len() {
                for j in (i+1)..positions.len() {
                    // Edge collision: agent i was where j is now, and j was where i is now
                    if prev_positions[i] == positions[j] && prev_positions[j] == positions[i] {
                        panic!(
                            "Edge collision between agents {} and {} at timestep {}: swapped positions",
                            i, j, t
                        );
                    }
                }
            }
        }
    }
}
