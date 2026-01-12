pub mod auth;
pub mod leaderboard;
pub mod solver;

use crate::{config::Config, db::Database};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Database,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        Self { config, db }
    }
}
