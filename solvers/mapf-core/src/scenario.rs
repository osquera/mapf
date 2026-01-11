//! MovingAI `.scen` (scenario) format parser.

use thiserror::Error;

/// Errors from parsing a MovingAI scenario file.
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("missing version header")]
    MissingVersion,

    #[error("invalid version: {0}")]
    InvalidVersion(String),

    #[error("malformed entry on line {line}: {reason}")]
    MalformedEntry { line: usize, reason: String },
}

/// A single entry (agent task) in a scenario file.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioEntry {
    /// Bucket index (used for grouping by difficulty).
    pub bucket: u32,
    /// Name of the map file this entry refers to.
    pub map_name: String,
    /// Map width (for validation).
    pub map_width: u32,
    /// Map height (for validation).
    pub map_height: u32,
    /// Start x coordinate.
    pub start_x: u32,
    /// Start y coordinate.
    pub start_y: u32,
    /// Goal x coordinate.
    pub goal_x: u32,
    /// Goal y coordinate.
    pub goal_y: u32,
    /// Optimal path length (for validation/scoring).
    pub optimal_length: f64,
}

/// A parsed MovingAI scenario file.
#[derive(Debug, Clone)]
pub struct Scenario {
    version: u32,
    entries: Vec<ScenarioEntry>,
}

impl Scenario {
    /// Parse a `.scen` file content.
    ///
    /// Expected format:
    /// ```text
    /// version N
    /// bucket\tmap\twidth\theight\tstart_x\tstart_y\tgoal_x\tgoal_y\toptimal
    /// ...
    /// ```
    pub fn parse(input: &str) -> Result<Self, ScenarioError> {
        let mut lines = input.lines().enumerate();

        // First non-empty line should be "version N"
        let version = loop {
            match lines.next() {
                Some((_, line)) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(rest) = trimmed.strip_prefix("version ") {
                        break rest
                            .trim()
                            .parse::<u32>()
                            .map_err(|_| ScenarioError::InvalidVersion(rest.to_string()))?;
                    } else {
                        return Err(ScenarioError::MissingVersion);
                    }
                }
                None => return Err(ScenarioError::MissingVersion),
            }
        };

        let mut entries = Vec::new();

        for (line_no, line) in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() < 9 {
                return Err(ScenarioError::MalformedEntry {
                    line: line_no + 1,
                    reason: format!("expected 9 columns, got {}", parts.len()),
                });
            }

            let parse_u32 = |idx: usize, name: &str| -> Result<u32, ScenarioError> {
                parts[idx].parse().map_err(|_| ScenarioError::MalformedEntry {
                    line: line_no + 1,
                    reason: format!("invalid {}: {}", name, parts[idx]),
                })
            };

            let parse_f64 = |idx: usize, name: &str| -> Result<f64, ScenarioError> {
                parts[idx].parse().map_err(|_| ScenarioError::MalformedEntry {
                    line: line_no + 1,
                    reason: format!("invalid {}: {}", name, parts[idx]),
                })
            };

            entries.push(ScenarioEntry {
                bucket: parse_u32(0, "bucket")?,
                map_name: parts[1].to_string(),
                map_width: parse_u32(2, "width")?,
                map_height: parse_u32(3, "height")?,
                start_x: parse_u32(4, "start_x")?,
                start_y: parse_u32(5, "start_y")?,
                goal_x: parse_u32(6, "goal_x")?,
                goal_y: parse_u32(7, "goal_y")?,
                optimal_length: parse_f64(8, "optimal_length")?,
            });
        }

        Ok(Self { version, entries })
    }

    /// Scenario file version number.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// All scenario entries (agent tasks).
    pub fn entries(&self) -> &[ScenarioEntry] {
        &self.entries
    }

    /// Extract (start, goal) coordinate pairs for all agents.
    /// Returns (starts, goals) where each is a Vec of (x, y).
    pub fn agents(&self) -> (Vec<(u32, u32)>, Vec<(u32, u32)>) {
        let starts = self.entries.iter().map(|e| (e.start_x, e.start_y)).collect();
        let goals = self.entries.iter().map(|e| (e.goal_x, e.goal_y)).collect();
        (starts, goals)
    }
}
