//! WASM bindings for the A* solver.

use js_sys::Uint32Array;
use wasm_bindgen::prelude::*;
use crate::astar::{solve_mapf_centralized_grid, Grid};

/// MAPF Solver that holds a pre-parsed grid for efficient reuse.
#[wasm_bindgen]
pub struct MapfSolver {
    grid: Grid,
}

#[wasm_bindgen]
impl MapfSolver {
    /// Initialize the solver with map data once.
    /// This parses the grid and keeps it in WASM memory for reuse.
    #[wasm_bindgen(constructor)]
    pub fn new(map_data: &[u8], width: u32, height: u32) -> Result<MapfSolver, JsError> {
        if map_data.len() != (width * height) as usize {
            return Err(JsError::new("Map data length doesn't match width*height"));
        }
        
        let grid = Grid::from_raw(map_data, width, height);
        Ok(MapfSolver { grid })
    }
    
    /// Get the map width.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.grid.width
    }
    
    /// Get the map height.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.grid.height
    }
    
    /// Solve for a specific set of agents on the pre-loaded map.
    pub fn solve(&self, starts: &[u32], goals: &[u32]) -> Result<WasmSolution, JsError> {
        // Validate input
        if starts.len() != goals.len() || starts.len() % 2 != 0 {
            return Err(JsError::new("starts and goals must have same length and be pairs"));
        }
        
        let num_agents = starts.len() / 2;
        
        // Build agent list: (start, goal) pairs
        let agents: Vec<((u32, u32), (u32, u32))> = (0..num_agents)
            .map(|i| {
                let start = (starts[i * 2], starts[i * 2 + 1]);
                let goal = (goals[i * 2], goals[i * 2 + 1]);
                (start, goal)
            })
            .collect();
        
        // Solve using the pre-parsed grid
        match solve_mapf_centralized_grid(&self.grid, &agents) {
            Some(paths) => {
                let mut all_paths: Vec<u32> = Vec::new();
                let mut total_cost: u32 = 0;
                
                for path in &paths {
                    // Store path length followed by coordinates
                    all_paths.push(path.steps.len() as u32);
                    for coord in &path.steps {
                        all_paths.push(coord.x);
                        all_paths.push(coord.y);
                    }
                    total_cost += path.cost();
                }
                
                Ok(WasmSolution {
                    paths_vec: all_paths,
                    cost: total_cost,
                    nodes_expanded: 0, // Not tracked in current implementation
                })
            }
            None => Err(JsError::new("Failed to find collision-free paths for all agents")),
        }
    }
}

/// Result of solving a MAPF instance (WASM-friendly).
#[wasm_bindgen]
pub struct WasmSolution {
    /// Flattened paths: [path1_len, x1, y1, x2, y2, ..., path2_len, ...]
    paths_vec: Vec<u32>,
    /// Total cost (sum of path lengths - 1 for each agent)
    cost: u32,
    /// Nodes expanded during search
    nodes_expanded: u64,
}

#[wasm_bindgen]
impl WasmSolution {
    /// Get paths as a Uint32Array for efficient JS interop.
    /// Format: [path1_len, x1, y1, x2, y2, ..., path2_len, ...]
    #[wasm_bindgen(getter)]
    pub fn paths(&self) -> Uint32Array {
        Uint32Array::from(&self.paths_vec[..])
    }

    #[wasm_bindgen(getter)]
    pub fn cost(&self) -> u32 {
        self.cost
    }

    #[wasm_bindgen(getter)]
    pub fn nodes_expanded(&self) -> u64 {
        self.nodes_expanded
    }
}

/// Solver information.
#[wasm_bindgen]
pub fn solver_info() -> String {
    "A* Reference Solver v0.2.0 (Multi-Agent with Prioritized Planning)".to_string()
}

// Legacy function for backward compatibility
/// Solve a MAPF instance (one-shot, creates solver internally).
#[wasm_bindgen]
pub fn solve(
    map_data: &[u8],
    width: u32,
    height: u32,
    starts: &[u32],
    goals: &[u32],
) -> Result<WasmSolution, JsError> {
    let solver = MapfSolver::new(map_data, width, height)?;
    solver.solve(starts, goals)
}
