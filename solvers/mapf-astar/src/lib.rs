//! Reference A* solver for single-agent and multi-agent pathfinding.
//!
//! This crate provides a baseline solver for the MAPF arena.
//! 
//! ## Features
//! - Single-agent A* pathfinding
//! - Multi-agent MAPF with step-by-step prioritized planning
//! - Grid struct for efficient map storage and reuse
//! - Cardinal movement only (no diagonals)

mod astar;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use astar::{astar_single, solve_mapf, solve_mapf_grid, solve_mapf_centralized, solve_mapf_centralized_grid, Coordinate, Grid, Path};
