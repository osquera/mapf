use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{db::LeaderboardEntry, error::Result};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub map_name: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total: usize,
}

/// GET /api/leaderboard
/// Retrieve leaderboard entries
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>> {
    let limit = query.limit.min(1000).max(1);

    let entries = state
        .db
        .get_leaderboard(query.map_name.as_deref(), limit)
        .await?;

    let total = entries.len();

    Ok(Json(LeaderboardResponse { entries, total }))
}
