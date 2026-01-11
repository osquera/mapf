//! Core MAPF types and MovingAI format parser.
//!
//! Provides data structures for grid maps and scenarios,
//! plus parsers for the MovingAI `.map` and `.scen` formats.

mod map;
mod scenario;

pub use map::{GridMap, MapError, Tile};
pub use scenario::{Scenario, ScenarioEntry, ScenarioError};
