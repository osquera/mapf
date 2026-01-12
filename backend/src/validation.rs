/// Path validation for MAPF solutions - Rust port of validation.ts
/// Ensures solvers follow the rules: cardinal moves only, no collisions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub steps: Vec<Coordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub paths: Vec<Path>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u8>, // 0 = blocked, 1 = passable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationErrorType {
    DiagonalMove,
    OutOfBounds,
    BlockedCell,
    InvalidStart,
    InvalidGoal,
    VertexCollision,
    EdgeCollision,
    EmptyPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    #[serde(rename = "type")]
    pub error_type: ValidationErrorType,
    pub agent_index: usize,
    pub timestep: Option<usize>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

/// Check if a move is cardinal (N/S/E/W only, no diagonals).
pub fn is_cardinal_move(from: &Coordinate, to: &Coordinate) -> bool {
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    // Valid: (1,0), (0,1), or (0,0) for wait
    (dx == 1 && dy == 0) || (dx == 0 && dy == 1) || (dx == 0 && dy == 0)
}

/// Validate that a single path uses only cardinal moves.
pub fn validate_path_cardinal(path: &Path, agent_index: usize) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if path.steps.is_empty() {
        errors.push(ValidationError {
            error_type: ValidationErrorType::EmptyPath,
            agent_index,
            timestep: None,
            details: format!("Agent {} has empty path", agent_index),
        });
        return errors;
    }

    for t in 0..path.steps.len() - 1 {
        let from = &path.steps[t];
        let to = &path.steps[t + 1];

        if !is_cardinal_move(from, to) {
            errors.push(ValidationError {
                error_type: ValidationErrorType::DiagonalMove,
                agent_index,
                timestep: Some(t),
                details: format!(
                    "Agent {} made diagonal move from ({},{}) to ({},{}) at timestep {}",
                    agent_index, from.x, from.y, to.x, to.y, t
                ),
            });
        }
    }

    errors
}

/// Validate that a path stays within map bounds and on passable cells.
pub fn validate_path_on_map(
    path: &Path,
    agent_index: usize,
    map: &GridMap,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (t, pos) in path.steps.iter().enumerate() {
        // Bounds check
        if pos.x < 0 || pos.x >= map.width as i32 || pos.y < 0 || pos.y >= map.height as i32 {
            errors.push(ValidationError {
                error_type: ValidationErrorType::OutOfBounds,
                agent_index,
                timestep: Some(t),
                details: format!(
                    "Agent {} at ({},{}) is out of bounds at timestep {}",
                    agent_index, pos.x, pos.y, t
                ),
            });
            continue;
        }

        // Passable check
        let idx = (pos.y as u32 * map.width + pos.x as u32) as usize;
        if idx < map.tiles.len() && map.tiles[idx] == 0 {
            errors.push(ValidationError {
                error_type: ValidationErrorType::BlockedCell,
                agent_index,
                timestep: Some(t),
                details: format!(
                    "Agent {} at ({},{}) is on blocked cell at timestep {}",
                    agent_index, pos.x, pos.y, t
                ),
            });
        }
    }

    errors
}

/// Validate that paths don't have vertex collisions (two agents at same cell at same time).
pub fn validate_no_vertex_collisions(paths: &[Path]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Find max timestep
    let max_t = paths.iter().map(|p| p.steps.len()).max().unwrap_or(0);

    for t in 0..max_t {
        // Map of position -> agent index at this timestep
        let mut occupied = std::collections::HashMap::new();

        for (agent, path) in paths.iter().enumerate() {
            // If path ended, agent stays at last position
            let pos = if t < path.steps.len() {
                &path.steps[t]
            } else if !path.steps.is_empty() {
                &path.steps[path.steps.len() - 1]
            } else {
                continue;
            };

            let key = (pos.x, pos.y);

            if let Some(&other_agent) = occupied.get(&key) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::VertexCollision,
                    agent_index: agent,
                    timestep: Some(t),
                    details: format!(
                        "Agents {} and {} collide at ({},{}) at timestep {}",
                        other_agent, agent, pos.x, pos.y, t
                    ),
                });
            } else {
                occupied.insert(key, agent);
            }
        }
    }

    errors
}

/// Validate that paths don't have edge collisions (two agents swapping positions).
pub fn validate_no_edge_collisions(paths: &[Path]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Find max timestep
    let max_t = paths.iter().map(|p| p.steps.len()).max().unwrap_or(0);

    for t in 0..max_t.saturating_sub(1) {
        for i in 0..paths.len() {
            for j in (i + 1)..paths.len() {
                let path_i = &paths[i];
                let path_j = &paths[j];

                // Get positions at t and t+1 for both agents
                let pos_i_t = if t < path_i.steps.len() {
                    &path_i.steps[t]
                } else if !path_i.steps.is_empty() {
                    &path_i.steps[path_i.steps.len() - 1]
                } else {
                    continue;
                };

                let pos_i_t1 = if t + 1 < path_i.steps.len() {
                    &path_i.steps[t + 1]
                } else if !path_i.steps.is_empty() {
                    &path_i.steps[path_i.steps.len() - 1]
                } else {
                    continue;
                };

                let pos_j_t = if t < path_j.steps.len() {
                    &path_j.steps[t]
                } else if !path_j.steps.is_empty() {
                    &path_j.steps[path_j.steps.len() - 1]
                } else {
                    continue;
                };

                let pos_j_t1 = if t + 1 < path_j.steps.len() {
                    &path_j.steps[t + 1]
                } else if !path_j.steps.is_empty() {
                    &path_j.steps[path_j.steps.len() - 1]
                } else {
                    continue;
                };

                // Check if they swap (edge collision)
                if pos_i_t.x == pos_j_t1.x
                    && pos_i_t.y == pos_j_t1.y
                    && pos_j_t.x == pos_i_t1.x
                    && pos_j_t.y == pos_i_t1.y
                {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::EdgeCollision,
                        agent_index: i,
                        timestep: Some(t),
                        details: format!(
                            "Agents {} and {} swap positions between timesteps {} and {}",
                            i,
                            j,
                            t,
                            t + 1
                        ),
                    });
                }
            }
        }
    }

    errors
}

/// Validate that paths start and end at the correct positions.
pub fn validate_starts_and_goals(
    paths: &[Path],
    starts: &[Coordinate],
    goals: &[Coordinate],
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if path.steps.is_empty() {
            continue;
        }

        let path_start = &path.steps[0];
        let path_end = &path.steps[path.steps.len() - 1];
        let expected_start = &starts[i];
        let expected_goal = &goals[i];

        if path_start.x != expected_start.x || path_start.y != expected_start.y {
            errors.push(ValidationError {
                error_type: ValidationErrorType::InvalidStart,
                agent_index: i,
                timestep: Some(0),
                details: format!(
                    "Agent {} path starts at ({},{}) but should start at ({},{})",
                    i, path_start.x, path_start.y, expected_start.x, expected_start.y
                ),
            });
        }

        if path_end.x != expected_goal.x || path_end.y != expected_goal.y {
            errors.push(ValidationError {
                error_type: ValidationErrorType::InvalidGoal,
                agent_index: i,
                timestep: Some(path.steps.len() - 1),
                details: format!(
                    "Agent {} path ends at ({},{}) but should end at ({},{})",
                    i, path_end.x, path_end.y, expected_goal.x, expected_goal.y
                ),
            });
        }
    }

    errors
}

/// Fully validate a MAPF solution.
///
/// Checks:
/// 1. All moves are cardinal (N/S/E/W) or wait
/// 2. All positions are within bounds and on passable cells
/// 3. Paths start and end at correct positions
/// 4. No vertex collisions (two agents at same cell)
/// 5. No edge collisions (two agents swapping)
pub fn validate_solution(
    solution: &Solution,
    map: &GridMap,
    starts: &[Coordinate],
    goals: &[Coordinate],
) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate each path individually
    for (i, path) in solution.paths.iter().enumerate() {
        errors.extend(validate_path_cardinal(path, i));
        errors.extend(validate_path_on_map(path, i, map));
    }

    // Validate starts and goals
    errors.extend(validate_starts_and_goals(&solution.paths, starts, goals));

    // Validate collisions between agents
    if solution.paths.len() > 1 {
        errors.extend(validate_no_vertex_collisions(&solution.paths));
        errors.extend(validate_no_edge_collisions(&solution.paths));
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinal_moves() {
        assert!(is_cardinal_move(&Coordinate { x: 0, y: 0 }, &Coordinate { x: 1, y: 0 }));
        assert!(is_cardinal_move(&Coordinate { x: 0, y: 0 }, &Coordinate { x: 0, y: 1 }));
        assert!(is_cardinal_move(&Coordinate { x: 0, y: 0 }, &Coordinate { x: 0, y: 0 }));
        assert!(!is_cardinal_move(&Coordinate { x: 0, y: 0 }, &Coordinate { x: 1, y: 1 }));
    }

    #[test]
    fn test_empty_path() {
        let path = Path { steps: vec![] };
        let errors = validate_path_cardinal(&path, 0);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].error_type, ValidationErrorType::EmptyPath));
    }

    #[test]
    fn test_diagonal_move() {
        let path = Path {
            steps: vec![
                Coordinate { x: 0, y: 0 },
                Coordinate { x: 1, y: 1 }, // Diagonal!
            ],
        };
        let errors = validate_path_cardinal(&path, 0);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].error_type, ValidationErrorType::DiagonalMove));
    }
}
