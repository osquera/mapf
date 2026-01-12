use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{generate_api_key, hash_api_key},
    error::{AppError, Result},
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub key_name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub api_key: String,
    pub message: String,
}

/// POST /api/auth/register
/// Create a new user and generate an API key
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>> {
    // Validate inputs
    if req.username.is_empty() || req.email.is_empty() {
        return Err(AppError::BadRequest(
            "Username and email are required".to_string(),
        ));
    }

    if !req.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // Check if username already exists
    if let Some(_) = state.db.get_user_by_username(&req.username).await? {
        return Err(AppError::BadRequest(
            "Username already exists".to_string(),
        ));
    }

    // Create user
    let user = state
        .db
        .create_user(&req.username, &req.email)
        .await?;

    // Generate API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key)?;

    // Store API key
    state
        .db
        .create_api_key(user.id, &key_hash, &req.key_name)
        .await?;

    tracing::info!("Created user {} with API key", req.username);

    Ok(Json(RegisterResponse {
        user_id: user.id.to_string(),
        api_key: api_key.clone(),
        message: format!(
            "User created successfully. Save your API key: {}",
            api_key
        ),
    }))
}
