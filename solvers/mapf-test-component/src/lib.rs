use wit_bindgen::generate;

generate!({
    path: "../wit/mapf-solver.wit",
    world: "mapf-solver",
});

use exports::mapf::solver::solver::Guest;
// Types are generated in the root module for the package interfaces
use mapf::solver::types::{Coordinate, Path, Solution, Stats};

struct Component;

impl Guest for Component {
    fn solve(
        _map_data: Vec<u8>,
        _width: u32,
        _height: u32,
        starts: Vec<Coordinate>,
        goals: Vec<Coordinate>,
    ) -> Result<Solution, String> {
        // Simple "Wait at start" solver for testing
        // For each agent, just return a path containing the start position
        
        let mut paths = Vec::new();
        
        for (i, start) in starts.iter().enumerate() {
            // Check if goal is reachable (trivial check)
            if i >= goals.len() {
                return Err("More starts than goals".to_string());
            }

            // Just stay at start for 1 step
            paths.push(Path {
                steps: vec![
                    Coordinate { x: start.x, y: start.y },
                    Coordinate { x: start.x, y: start.y } // Wait
                ],
            });
        }

        Ok(Solution {
            paths,
            cost: 0,
        })
    }

    fn get_stats() -> Option<Stats> {
        Some(Stats {
            nodes_expanded: 0,
            time_us: 0,
        })
    }

    fn info() -> String {
        "Test Component Solver v0.1".to_string()
    }
}

export!(Component);

