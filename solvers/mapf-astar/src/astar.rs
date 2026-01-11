//! A* pathfinding implementation with multi-agent support.
//!
//! Uses prioritized step-by-step planning for multi-agent pathfinding (MAPF):
//! - At each timestep, agents choose actions in priority order
//! - If an action causes a conflict, the agent tries alternatives
//! - This avoids vertex collisions (same cell at same time)
//! - This avoids edge collisions (agents swapping positions)

/// Centralized A* MAPF solver using Grid
pub fn solve_mapf_centralized_grid(
    grid: &Grid,
    agents: &[((u32, u32), (u32, u32))],
) -> Option<Vec<Path>> {
    let num_agents = agents.len();
    let starts: Vec<Coordinate> = agents.iter().map(|&((sx, sy), _)| Coordinate { x: sx, y: sy }).collect();
    let goals: Vec<Coordinate> = agents.iter().map(|&(_, (gx, gy))| Coordinate { x: gx, y: gy }).collect();

    // Initial state
    let start_state = GlobalState {
        positions: starts.clone(),
        paths: starts.iter().map(|&p| vec![p]).collect(),
        cost: 0,
        timestep: 0,
        goals: goals.clone(),
    };

    // Priority queue (min-heap)
    let mut open = BinaryHeap::new();
    open.push(start_state);

    // Visited set: (positions, timestep)
    let mut visited: HashSet<(Vec<Coordinate>, u32)> = HashSet::new();
    visited.insert((starts.clone(), 0));

    // Main search loop
    while let Some(state) = open.pop() {
        // Check if all agents reached their goals
        if state.positions.iter().zip(state.goals.iter()).all(|(p, g)| p == g) {
            // Return solution paths
            return Some(state.paths.into_iter().map(|steps| Path { steps }).collect());
        }

        // Generate all possible moves for each agent (including wait)
        let mut moves_per_agent: Vec<Vec<Coordinate>> = Vec::with_capacity(num_agents);
        for i in 0..num_agents {
            let mut moves = Vec::new();
            // Cardinal moves
            for (neighbor, _) in neighbors_grid(state.positions[i], grid) {
                moves.push(neighbor);
            }
            // Wait (NoOp)
            moves.push(state.positions[i]);
            moves_per_agent.push(moves);
        }

        // Generate all joint moves (cartesian product)
        let mut joint_moves = vec![];
        fn backtrack(
            moves_per_agent: &Vec<Vec<Coordinate>>,
            current: &mut Vec<Coordinate>,
            idx: usize,
            joint_moves: &mut Vec<Vec<Coordinate>>,
        ) {
            if idx == moves_per_agent.len() {
                joint_moves.push(current.clone());
                return;
            }
            for &m in &moves_per_agent[idx] {
                current.push(m);
                backtrack(moves_per_agent, current, idx + 1, joint_moves);
                current.pop();
            }
        }
        backtrack(&moves_per_agent, &mut Vec::with_capacity(num_agents), 0, &mut joint_moves);

        // For each joint move, check for conflicts and expand
        for next_positions in joint_moves {
            // Vertex conflict: two agents in same cell
            let mut unique = HashSet::new();
            if !next_positions.iter().all(|p| unique.insert(*p)) {
                continue; // skip joint move with vertex conflict
            }
            // Edge conflict: agents swap positions
            let mut edge_conflict = false;
            for i in 0..num_agents {
                for j in (i+1)..num_agents {
                    if state.positions[i] == next_positions[j] && state.positions[j] == next_positions[i] {
                        edge_conflict = true;
                        break;
                    }
                }
                if edge_conflict { break; }
            }
            if edge_conflict { continue; }

            // Check if already visited
            let visit_key = (next_positions.clone(), state.timestep + 1);
            if visited.contains(&visit_key) {
                continue;
            }
            visited.insert(visit_key);

            // Build new paths
            let mut new_paths = state.paths.clone();
            for i in 0..num_agents {
                new_paths[i].push(next_positions[i]);
            }

            // Cost: +1 per agent move (wait counts as move)
            let new_cost = state.cost + 1;
            let new_state = GlobalState {
                positions: next_positions,
                paths: new_paths,
                cost: new_cost,
                timestep: state.timestep + 1,
                goals: state.goals.clone(),
            };
            open.push(new_state);
        }
    }
    None
}

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A 2D coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

/// A grid map for pathfinding.
/// Stores the map data and dimensions for efficient reuse.
#[derive(Debug, Clone)]
pub struct Grid {
    /// Map data: 1 = passable, 0 = blocked (row-major order)
    data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Grid {
    /// Create a new Grid from raw map data.
    pub fn from_raw(map_data: &[u8], width: u32, height: u32) -> Self {
        Self {
            data: map_data.to_vec(),
            width,
            height,
        }
    }
    
    /// Check if a coordinate is passable.
    #[inline]
    pub fn is_passable(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let idx = (y * self.width + x) as usize;
        self.data.get(idx).copied() == Some(1)
    }
    
    /// Check if a coordinate is within bounds.
    #[inline]
    pub fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }
    
    /// Get the raw data slice.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// A path from start to goal.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub steps: Vec<Coordinate>,
}

impl Path {
    /// Calculate path cost (number of cardinal moves).
    /// Each move (N/S/E/W) has cost 1. NoOp (wait) also has cost 1.
    pub fn cost(&self) -> u32 {
        if self.steps.len() <= 1 {
            0
        } else {
            (self.steps.len() - 1) as u32
        }
    }
    
    /// Validate that the path uses only cardinal moves or NoOp (no diagonals).
    /// NoOp is when an agent stays in the same position (wait action).
    pub fn is_valid_cardinal(&self) -> bool {
        for window in self.steps.windows(2) {
            let dx = (window[1].x as i32 - window[0].x as i32).abs();
            let dy = (window[1].y as i32 - window[0].y as i32).abs();
            // Each step must be:
            // - exactly 1 in one direction, 0 in the other (cardinal move), OR
            // - 0 in both directions (NoOp/wait)
            let is_cardinal = (dx == 1 && dy == 0) || (dx == 0 && dy == 1);
            let is_noop = dx == 0 && dy == 0;
            if !is_cardinal && !is_noop {
                return false;
            }
        }
        true
    }
}

/// Node for A* priority queue.
#[derive(Clone, Eq, PartialEq)]
struct Node {
    coord: Coordinate,
    g_cost: u32, // cost from start
    f_cost: u32, // g + heuristic
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Manhattan distance heuristic.
fn heuristic(from: Coordinate, to: Coordinate) -> u32 {
    let dx = (from.x as i32 - to.x as i32).unsigned_abs();
    let dy = (from.y as i32 - to.y as i32).unsigned_abs();
    dx + dy
}

/// Get valid neighbors (4-connected: North, South, East, West only).
/// MAPF requires cardinal movement only - no diagonal moves allowed.
fn neighbors_grid(coord: Coordinate, grid: &Grid) -> Vec<(Coordinate, u32)> {
    let mut result = Vec::with_capacity(4);
    let (x, y) = (coord.x as i32, coord.y as i32);
    let w = grid.width as i32;
    let h = grid.height as i32;

    // Cardinal directions only: North, South, West, East (cost 1 each)
    let cardinals = [
        (0, -1),  // North
        (0, 1),   // South
        (-1, 0),  // West
        (1, 0),   // East
    ];
    
    for (dx, dy) in cardinals {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0 && nx < w && ny >= 0 && ny < h {
            let ux = nx as u32;
            let uy = ny as u32;
            if grid.is_passable(ux, uy) {
                result.push((Coordinate { x: ux, y: uy }, 1));
            }
        }
    }

    result
}

/// Get valid neighbors using raw map data (for backward compatibility).
fn neighbors(coord: Coordinate, width: u32, height: u32, map: &[u8]) -> Vec<(Coordinate, u32)> {
    let mut result = Vec::with_capacity(4);
    let (x, y) = (coord.x as i32, coord.y as i32);
    let w = width as i32;
    let h = height as i32;

    // Cardinal directions only: North, South, West, East (cost 1 each)
    // No diagonal movement allowed in standard MAPF
    let cardinals = [
        (0, -1),  // North
        (0, 1),   // South
        (-1, 0),  // West
        (1, 0),   // East
    ];
    
    for (dx, dy) in cardinals {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0 && nx < w && ny >= 0 && ny < h {
            let idx = (ny as u32 * width + nx as u32) as usize;
            if map[idx] != 0 {
                result.push((Coordinate { x: nx as u32, y: ny as u32 }, 1));
            }
        }
    }

    result
}

/// Find a path for a single agent using A*.
///
/// - `map`: Flat byte array (row-major). 1 = passable, 0 = blocked.
/// - `width`, `height`: Map dimensions.
/// - `start`: (x, y) start position.
/// - `goal`: (x, y) goal position.
///
/// Returns `Some(Path)` if found, `None` if no path exists.
pub fn astar_single(
    map: &[u8],
    width: u32,
    height: u32,
    start: (u32, u32),
    goal: (u32, u32),
) -> Option<Path> {
    let start = Coordinate { x: start.0, y: start.1 };
    let goal = Coordinate { x: goal.0, y: goal.1 };

    // Bounds check
    if start.x >= width || start.y >= height || goal.x >= width || goal.y >= height {
        return None;
    }

    // Check start/goal passability
    let start_idx = (start.y * width + start.x) as usize;
    let goal_idx = (goal.y * width + goal.x) as usize;
    if map.get(start_idx).copied() != Some(1) || map.get(goal_idx).copied() != Some(1) {
        return None;
    }

    // Same start and goal
    if start == goal {
        return Some(Path { steps: vec![start] });
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<Coordinate, Coordinate> = HashMap::new();
    let mut g_score: HashMap<Coordinate, u32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Node {
        coord: start,
        g_cost: 0,
        f_cost: heuristic(start, goal),
    });

    while let Some(current) = open.pop() {
        if current.coord == goal {
            // Reconstruct path
            let mut path = vec![goal];
            let mut curr = goal;
            while let Some(&prev) = came_from.get(&curr) {
                path.push(prev);
                curr = prev;
            }
            path.reverse();
            return Some(Path { steps: path });
        }

        let current_g = g_score[&current.coord];

        for (neighbor, move_cost) in neighbors(current.coord, width, height, map) {
            let tentative_g = current_g + move_cost;

            if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, current.coord);
                g_score.insert(neighbor, tentative_g);
                open.push(Node {
                    coord: neighbor,
                    g_cost: tentative_g,
                    f_cost: tentative_g + heuristic(neighbor, goal),
                });
            }
        }
    }

    None // No path found
}

/// Action types an agent can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Move to an adjacent cell (N/S/E/W)
    Move,
    /// Stay in place (wait)
    NoOp,
}

/// An action an agent can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Action {
    action_type: ActionType,
    from: Coordinate,
    target: Coordinate,
    priority: u32, // lower is better (distance to goal)
}

/// Get prioritized actions for an agent at a position.
/// Actions are sorted by how much they reduce distance to goal.
/// Includes both Move actions (cardinal directions) and NoOp (wait).
fn get_prioritized_actions(
    pos: Coordinate,
    goal: Coordinate,
    width: u32,
    height: u32,
    map: &[u8],
) -> Vec<Action> {
    let mut actions = Vec::with_capacity(5);
    
    // Add cardinal Move actions
    for (neighbor, _) in neighbors(pos, width, height, map) {
        actions.push(Action {
            action_type: ActionType::Move,
            from: pos,
            target: neighbor,
            priority: heuristic(neighbor, goal),
        });
    }
    
    // Add NoOp action (stay in place / wait)
    actions.push(Action {
        action_type: ActionType::NoOp,
        from: pos,
        target: pos,
        priority: heuristic(pos, goal) + 1, // slightly worse than moving toward goal
    });
    
    // Sort by priority (lower distance to goal = better)
    actions.sort_by_key(|a| a.priority);
    
    actions
}

/// Get prioritized actions using Grid.
fn get_prioritized_actions_grid(
    pos: Coordinate,
    goal: Coordinate,
    grid: &Grid,
) -> Vec<Action> {
    let mut actions = Vec::with_capacity(5);
    
    // Add cardinal Move actions
    for (neighbor, _) in neighbors_grid(pos, grid) {
        actions.push(Action {
            action_type: ActionType::Move,
            from: pos,
            target: neighbor,
            priority: heuristic(neighbor, goal),
        });
    }
    
    // Add NoOp action (stay in place / wait)
    actions.push(Action {
        action_type: ActionType::NoOp,
        from: pos,
        target: pos,
        priority: heuristic(pos, goal) + 1,
    });
    
    actions.sort_by_key(|a| a.priority);
    actions
}

/// Check for conflicts between two agents' actions.
/// Returns true if there is a conflict.
/// 
/// Conflict types:
/// 1. Vertex conflict: Two agents move to the same cell
/// 2. Edge conflict: Two agents swap positions (A->B and B->A)
fn has_conflict(action_a: &Action, action_b: &Action) -> bool {
    // NoOp actions don't cause conflicts with movement
    // (but can cause vertex conflict if another agent moves INTO the NoOp agent's cell)
    
    // Vertex conflict: both agents want to be in the same cell
    if action_a.target == action_b.target {
        return true;
    }
    
    // Edge conflict (swap): agent A goes from X to Y, agent B goes from Y to X
    // Only check if both are actually moving (not NoOp)
    if action_a.action_type == ActionType::Move && action_b.action_type == ActionType::Move {
        if action_a.from == action_b.target && action_a.target == action_b.from {
            return true;
        }
    }
    
    false
}

/// Solve MAPF for multiple agents using step-by-step prioritized planning.
///
/// At each timestep:
/// 1. Each agent generates a priority queue of actions (Move + NoOp)
/// 2. Agents commit to actions in priority order (by agent index)
/// 3. If the best action conflicts, try the next action
/// 4. Continue until all agents reach their goals
///
/// Returns paths for all agents, or None if any agent gets stuck.
pub fn solve_mapf(
    map: &[u8],
    width: u32,
    height: u32,
    agents: &[((u32, u32), (u32, u32))], // (start, goal) pairs
) -> Option<Vec<Path>> {
    let num_agents = agents.len();
    
    // Current positions
    let mut positions: Vec<Coordinate> = agents
        .iter()
        .map(|&((sx, sy), _)| Coordinate { x: sx, y: sy })
        .collect();
    
    // Goals
    let goals: Vec<Coordinate> = agents
        .iter()
        .map(|&(_, (gx, gy))| Coordinate { x: gx, y: gy })
        .collect();
    
    // Paths being built
    let mut paths: Vec<Vec<Coordinate>> = positions.iter().map(|&p| vec![p]).collect();
    
    // Maximum timesteps to prevent infinite loops
    let max_timesteps: u32 = (width + height) * 1000;
    
    for _t in 0..max_timesteps {
        // Check if all agents have reached their goals
        let all_done: bool = positions.iter().zip(goals.iter()).all(|(p, g)| p == g);
        if all_done {
            break;
        }
        
        // Committed actions for this timestep
        let mut committed_actions: Vec<Option<Action>> = vec![None; num_agents];
        
        // Process agents in priority order (by index)
        for i in 0..num_agents {
            
            // Get prioritized actions
            let actions = get_prioritized_actions(
                positions[i],
                goals[i],
                width,
                height,
                map,
            );
            
            // Try each action in priority order
            let mut found_action = false;
            for action in actions {
                // Check for conflicts with all previously committed actions
                let mut has_any_conflict = false;
                for j in 0..i {
                    if let Some(ref other_action) = committed_actions[j] {
                        if has_conflict(&action, other_action) {
                            has_any_conflict = true;
                            break;
                        }
                    }
                }
                
                if has_any_conflict {
                    continue;
                }
                
                // This action is valid - commit it
                committed_actions[i] = Some(action);
                found_action = true;
                break;
            }
            
            if !found_action {
                // Agent is stuck - no valid action available
                eprintln!("Agent {} stuck at ({}, {})", i, positions[i].x, positions[i].y);
                return None;
            }
        }
        
        // Apply all committed actions
        for i in 0..num_agents {
            if let Some(action) = committed_actions[i] {
                positions[i] = action.target;
                // Add to path (including NoOp for wait actions)
                let last = paths[i].last().unwrap();
                if action.target != *last || action.target != goals[i] {
                    paths[i].push(action.target);
                }
            }
        }
    }
    
    // Verify all agents reached their goals
    for (i, (pos, goal)) in positions.iter().zip(goals.iter()).enumerate() {
        if pos != goal {
            eprintln!("Agent {} did not reach goal", i);
            return None;
        }
    }
    
    Some(paths.into_iter().map(|steps| Path { steps }).collect())
}

/// Solve MAPF using a pre-parsed Grid (more efficient for multiple solves).
pub fn solve_mapf_grid(
    grid: &Grid,
    agents: &[((u32, u32), (u32, u32))],
) -> Option<Vec<Path>> {
    let num_agents = agents.len();
    
    // Current positions
    let mut positions: Vec<Coordinate> = agents
        .iter()
        .map(|&((sx, sy), _)| Coordinate { x: sx, y: sy })
        .collect();
    
    // Goals
    let goals: Vec<Coordinate> = agents
        .iter()
        .map(|&(_, (gx, gy))| Coordinate { x: gx, y: gy })
        .collect();
    
    // Paths being built
    let mut paths: Vec<Vec<Coordinate>> = positions.iter().map(|&p| vec![p]).collect();
    
    // Maximum timesteps to prevent infinite loops
    let max_timesteps = (grid.width + grid.height) * 4;
    
    for _t in 0..max_timesteps {
        // Check if all agents have reached their goals
        let all_done = positions.iter().zip(goals.iter()).all(|(p, g)| p == g);
        if all_done {
            break;
        }
        
        // Committed actions for this timestep
        let mut committed_actions: Vec<Option<Action>> = vec![None; num_agents];
        
        // Process agents in priority order (by index)
        for i in 0..num_agents {
            // Get prioritized actions
            let actions = get_prioritized_actions_grid(
                positions[i],
                goals[i],
                grid,
            );
            
            // Try each action in priority order
            let mut found_action = false;
            for action in actions {
                // Check for conflicts with all previously committed actions
                let mut has_any_conflict = false;
                for j in 0..i {
                    if let Some(ref other_action) = committed_actions[j] {
                        if has_conflict(&action, other_action) {
                            has_any_conflict = true;
                            break;
                        }
                    }
                }
                
                if has_any_conflict {
                    continue;
                }
                
                // This action is valid - commit it
                committed_actions[i] = Some(action);
                found_action = true;
                break;
            }
            
            if !found_action {
                eprintln!("Agent {} stuck at ({}, {})", i, positions[i].x, positions[i].y);
                return None;
            }
        }
        
        // Apply all committed actions
        for i in 0..num_agents {
            if let Some(action) = committed_actions[i] {
                positions[i] = action.target;
                let last = paths[i].last().unwrap();
                if action.target != *last || action.target != goals[i] {
                    paths[i].push(action.target);
                }
            }
        }
    }
    
    // Verify all agents reached their goals
    for (i, (pos, goal)) in positions.iter().zip(goals.iter()).enumerate() {
        if pos != goal {
            eprintln!("Agent {} did not reach goal", i);
            return None;
        }
    }
    
    Some(paths.into_iter().map(|steps| Path { steps }).collect())
}

/// Global state for centralized MAPF A*
#[derive(Clone, Eq, PartialEq, Hash)]
struct GlobalState {
    positions: Vec<Coordinate>, // Current positions of all agents
    paths: Vec<Vec<Coordinate>>, // Paths for all agents so far
    cost: u32, // Total cost so far
    timestep: u32, // Current timestep
    goals: Vec<Coordinate>, // Store goals for f_cost
}

// Implement ordering for BinaryHeap (min-heap by f_cost)
impl Ord for GlobalState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse for min-heap: lower f_cost is higher priority
        other.f_cost().cmp(&self.f_cost())
    }
}

impl PartialOrd for GlobalState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl GlobalState {
    /// Heuristic: sum of Manhattan distances to goals
    fn heuristic(&self) -> u32 {
        self.positions.iter().zip(self.goals.iter())
            .map(|(p, g)| heuristic(*p, *g))
            .sum()
    }
    /// Total estimated cost (g + h)
    fn f_cost(&self) -> u32 {
        self.cost + self.heuristic()
    }
}

/// Centralized A* MAPF solver
pub fn solve_mapf_centralized(
    map: &[u8],
    width: u32,
    height: u32,
    agents: &[((u32, u32), (u32, u32))],
) -> Option<Vec<Path>> {
    let num_agents = agents.len();
    let starts: Vec<Coordinate> = agents.iter().map(|&((sx, sy), _)| Coordinate { x: sx, y: sy }).collect();
    let goals: Vec<Coordinate> = agents.iter().map(|&(_, (gx, gy))| Coordinate { x: gx, y: gy }).collect();

    // Initial state
    let start_state = GlobalState {
        positions: starts.clone(),
        paths: starts.iter().map(|&p| vec![p]).collect(),
        cost: 0,
        timestep: 0,
        goals: goals.clone(),
    };

    // Priority queue (min-heap)
    let mut open = BinaryHeap::new();
    open.push(start_state);

    // Visited set: (positions, timestep)
    let mut visited: HashSet<(Vec<Coordinate>, u32)> = HashSet::new();
    visited.insert((starts.clone(), 0));

    // Main search loop
    while let Some(state) = open.pop() {
        // Check if all agents reached their goals
        if state.positions.iter().zip(state.goals.iter()).all(|(p, g)| p == g) {
            // Return solution paths
            return Some(state.paths.into_iter().map(|steps| Path { steps }).collect());
        }

        // Generate all possible moves for each agent (including wait)
        let mut moves_per_agent: Vec<Vec<Coordinate>> = Vec::with_capacity(num_agents);
        for i in 0..num_agents {
            let mut moves = Vec::new();
            // Cardinal moves
            for (neighbor, _) in neighbors(state.positions[i], width, height, map) {
                moves.push(neighbor);
            }
            // Wait (NoOp)
            moves.push(state.positions[i]);
            moves_per_agent.push(moves);
        }

        // Generate all joint moves (cartesian product)
        let mut joint_moves = vec![];
        fn backtrack(
            moves_per_agent: &Vec<Vec<Coordinate>>,
            current: &mut Vec<Coordinate>,
            idx: usize,
            joint_moves: &mut Vec<Vec<Coordinate>>,
        ) {
            if idx == moves_per_agent.len() {
                joint_moves.push(current.clone());
                return;
            }
            for &m in &moves_per_agent[idx] {
                current.push(m);
                backtrack(moves_per_agent, current, idx + 1, joint_moves);
                current.pop();
            }
        }
        backtrack(&moves_per_agent, &mut Vec::with_capacity(num_agents), 0, &mut joint_moves);

        // For each joint move, check for conflicts and expand
        for next_positions in joint_moves {
            // Vertex conflict: two agents in same cell
            let mut unique = HashSet::new();
            if !next_positions.iter().all(|p| unique.insert(*p)) {
                continue; // skip joint move with vertex conflict
            }
            // Edge conflict: agents swap positions
            let mut edge_conflict = false;
            for i in 0..num_agents {
                for j in (i+1)..num_agents {
                    if state.positions[i] == next_positions[j] && state.positions[j] == next_positions[i] {
                        edge_conflict = true;
                        break;
                    }
                }
                if edge_conflict { break; }
            }
            if edge_conflict { continue; }

            // Check if already visited
            let visit_key = (next_positions.clone(), state.timestep + 1);
            if visited.contains(&visit_key) {
                continue;
            }
            visited.insert(visit_key);

            // Build new paths
            let mut new_paths = state.paths.clone();
            for i in 0..num_agents {
                new_paths[i].push(next_positions[i]);
            }

            // Cost: +1 per agent move (wait counts as move)
            let new_cost = state.cost + 1;
            let new_state = GlobalState {
                positions: next_positions,
                paths: new_paths,
                cost: new_cost,
                timestep: state.timestep + 1,
                goals: state.goals.clone(),
            };
            open.push(new_state);
        }
    }
    None
}
