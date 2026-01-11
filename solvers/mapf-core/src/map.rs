//! MovingAI `.map` format parser.

use thiserror::Error;

/// A single cell in a grid map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    /// Agent can traverse this cell.
    Passable,
    /// Obstacle; agent cannot enter.
    Blocked,
}

/// Errors from parsing a MovingAI map file.
#[derive(Debug, Error)]
pub enum MapError {
    #[error("missing header field: {0}")]
    MissingHeader(&'static str),

    #[error("invalid header value for {field}: {value}")]
    InvalidHeader { field: &'static str, value: String },

    #[error("dimension mismatch: expected {expected} rows, got {got}")]
    DimensionMismatch { expected: u32, got: u32 },

    #[error("row {row} width mismatch: expected {expected}, got {got}")]
    RowWidthMismatch { row: u32, expected: u32, got: u32 },
}

/// A parsed grid map from the MovingAI format.
#[derive(Debug, Clone)]
pub struct GridMap {
    width: u32,
    height: u32,
    /// Row-major; index = y * width + x
    tiles: Vec<Tile>,
}

impl GridMap {
    /// Parse a MovingAI `.map` file content.
    ///
    /// Expected format:
    /// ```text
    /// type octile
    /// height N
    /// width M
    /// map
    /// <N lines of M characters>
    /// ```
    pub fn parse(input: &str) -> Result<Self, MapError> {
        let mut lines = input.lines();

        // Parse header
        let mut map_type: Option<&str> = None;
        let mut height: Option<u32> = None;
        let mut width: Option<u32> = None;

        for line in lines.by_ref() {
            let line = line.trim();
            if line.eq_ignore_ascii_case("map") {
                break;
            }
            if let Some(rest) = line.strip_prefix("type ") {
                map_type = Some(rest.trim());
            } else if let Some(rest) = line.strip_prefix("height ") {
                height = Some(
                    rest.trim()
                        .parse()
                        .map_err(|_| MapError::InvalidHeader {
                            field: "height",
                            value: rest.to_string(),
                        })?,
                );
            } else if let Some(rest) = line.strip_prefix("width ") {
                width = Some(
                    rest.trim()
                        .parse()
                        .map_err(|_| MapError::InvalidHeader {
                            field: "width",
                            value: rest.to_string(),
                        })?,
                );
            }
        }

        let _map_type = map_type.ok_or(MapError::MissingHeader("type"))?;
        let height = height.ok_or(MapError::MissingHeader("height"))?;
        let width = width.ok_or(MapError::MissingHeader("width"))?;

        // Parse grid rows
        let mut tiles = Vec::with_capacity((width * height) as usize);
        let mut row_count: u32 = 0;

        for line in lines {
            if row_count >= height {
                break; // ignore trailing lines
            }
            let chars: Vec<char> = line.chars().collect();
            if chars.len() != width as usize {
                return Err(MapError::RowWidthMismatch {
                    row: row_count,
                    expected: width,
                    got: chars.len() as u32,
                });
            }
            for ch in chars {
                tiles.push(Self::char_to_tile(ch));
            }
            row_count += 1;
        }

        if row_count < height {
            return Err(MapError::DimensionMismatch {
                expected: height,
                got: row_count,
            });
        }

        Ok(Self { width, height, tiles })
    }

    fn char_to_tile(ch: char) -> Tile {
        match ch {
            '.' | 'G' | 'S' => Tile::Passable,
            // Everything else is blocked (@ T O W etc.)
            _ => Tile::Blocked,
        }
    }

    /// Width of the map (x dimension).
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the map (y dimension).
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get tile at (x, y). Returns `None` if out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Option<Tile> {
        if x < self.width && y < self.height {
            Some(self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Check if cell (x, y) is passable (returns false for out-of-bounds).
    pub fn is_passable(&self, x: u32, y: u32) -> bool {
        self.get(x, y) == Some(Tile::Passable)
    }

    /// Convert to a flat byte array (row-major). 1 = passable, 0 = blocked.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.tiles
            .iter()
            .map(|t| match t {
                Tile::Passable => 1,
                Tile::Blocked => 0,
            })
            .collect()
    }

    /// Create from raw bytes (row-major, 1 = passable, 0 = blocked).
    pub fn from_bytes(width: u32, height: u32, data: &[u8]) -> Option<Self> {
        if data.len() != (width * height) as usize {
            return None;
        }
        let tiles = data
            .iter()
            .map(|&b| if b != 0 { Tile::Passable } else { Tile::Blocked })
            .collect();
        Some(Self { width, height, tiles })
    }
}
