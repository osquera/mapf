use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    auth::AuthenticatedUser,
    error::{AppError, Result},
    executor::WasmExecutor,
    validation::{self, Coordinate, GridMap},
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    #[serde(rename = "wasmBytes")]
    pub wasm_bytes: Vec<u8>,
    pub map: MapData,
    pub starts: Vec<Coordinate>,
    pub goals: Vec<Coordinate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub solution: Option<validation::Solution>,
    pub validation_errors: Vec<validation::ValidationError>,
    pub stats: ExecutionStats,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionStats {
    pub instruction_count: Option<u64>,
    pub execution_time_ms: u64,
    pub cost: Option<i64>,
    pub makespan: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    pub solver_name: String,
    pub map_name: String,
    pub scenario_id: String,
    #[serde(rename = "wasmBytes")]
    pub wasm_bytes: Vec<u8>,
    pub map: MapData,
    pub starts: Vec<Coordinate>,
    pub goals: Vec<Coordinate>,
}

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub submission_id: String,
    pub verification_id: String,
    pub message: String,
}

/// POST /api/verify
/// Verify a WASM solver without storing results (open endpoint for testing)
pub async fn verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>> {
    // Validate WASM size
    let max_size = state.config.max_wasm_size_mb * 1024 * 1024;
    if req.wasm_bytes.len() > max_size {
        return Err(AppError::BadRequest(format!(
            "WASM file too large: {} bytes (max: {} MB)",
            req.wasm_bytes.len(),
            state.config.max_wasm_size_mb
        )));
    }

    // Create executor
    let executor = WasmExecutor::new(
        state.config.solver_timeout_secs,
        state.config.solver_instruction_limit,
    )
    .map_err(|e| AppError::WasmExecution(format!("Failed to create executor: {}", e)))?;

    // Convert map
    let grid_map = GridMap {
        width: req.map.width,
        height: req.map.height,
        tiles: req.map.tiles,
    };

    // Execute solver
    let solver_result = executor
        .execute(&req.wasm_bytes, &grid_map, &req.starts, &req.goals)
        .await
        .map_err(|e| AppError::WasmExecution(format!("Execution failed: {}", e)))?;

    // If solver failed, return error
    if let Some(error) = &solver_result.error {
        return Ok(Json(VerifyResponse {
            valid: false,
            solution: None,
            validation_errors: vec![],
            stats: ExecutionStats {
                instruction_count: solver_result.stats.instruction_count,
                execution_time_ms: solver_result.stats.execution_time_ms,
                cost: None,
                makespan: None,
            },
            error: Some(error.clone()),
        }));
    }

    // Validate solution
    let solution = solver_result.solution.ok_or_else(|| {
        AppError::WasmExecution("Solver returned no solution and no error".to_string())
    })?;

    let validation_result =
        validation::validate_solution(&solution, &grid_map, &req.starts, &req.goals);

    // Calculate cost and makespan if valid
    let (cost, makespan) = if validation_result.valid {
        let cost: i64 = solution.paths.iter().map(|p| p.steps.len() as i64).sum();
        let makespan: i64 = solution
            .paths
            .iter()
            .map(|p| p.steps.len() as i64)
            .max()
            .unwrap_or(0);
        (Some(cost), Some(makespan))
    } else {
        (None, None)
    };

    Ok(Json(VerifyResponse {
        valid: validation_result.valid,
        solution: Some(solution),
        validation_errors: validation_result.errors,
        stats: ExecutionStats {
            instruction_count: solver_result.stats.instruction_count,
            execution_time_ms: solver_result.stats.execution_time_ms,
            cost,
            makespan,
        },
        error: None,
    }))
}

/// POST /api/submit
/// Submit a solver result to the leaderboard (requires authentication)
pub async fn submit(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<SubmitRequest>,
) -> Result<Json<SubmitResponse>> {
    // Validate inputs
    if req.solver_name.is_empty() || req.map_name.is_empty() || req.scenario_id.is_empty() {
        return Err(AppError::BadRequest(
            "solver_name, map_name, and scenario_id are required".to_string(),
        ));
    }

    // Hash WASM for deduplication
    let mut hasher = Sha256::new();
    hasher.update(&req.wasm_bytes);
    let wasm_hash = format!("{:x}", hasher.finalize());

    // Create submission record
    let submission = state
        .db
        .create_submission(auth.user_id, &req.solver_name, &wasm_hash)
        .await?;

    // Execute and validate (reuse verify logic)
    let executor = WasmExecutor::new(
        state.config.solver_timeout_secs,
        state.config.solver_instruction_limit,
    )
    .map_err(|e| AppError::WasmExecution(format!("Failed to create executor: {}", e)))?;

    let grid_map = GridMap {
        width: req.map.width,
        height: req.map.height,
        tiles: req.map.tiles,
    };

    let solver_result = executor
        .execute(&req.wasm_bytes, &grid_map, &req.starts, &req.goals)
        .await
        .map_err(|e| AppError::WasmExecution(format!("Execution failed: {}", e)))?;

    let valid = solver_result.error.is_none();
    let (cost, makespan, error_message) = if let Some(solution) = &solver_result.solution {
        let validation_result =
            validation::validate_solution(solution, &grid_map, &req.starts, &req.goals);

        if validation_result.valid {
            let cost: i64 = solution.paths.iter().map(|p| p.steps.len() as i64).sum();
            let makespan: i64 = solution
                .paths
                .iter()
                .map(|p| p.steps.len() as i64)
                .max()
                .unwrap_or(0);
            (Some(cost), Some(makespan), None)
        } else {
            let error_summary = validation_result
                .errors
                .iter()
                .map(|e| e.details.clone())
                .collect::<Vec<_>>()
                .join("; ");
            (None, None, Some(error_summary))
        }
    } else {
        (None, None, solver_result.error.clone())
    };

    // Store verification result
    let verification = state
        .db
        .create_verification_result(
            submission.id,
            &req.map_name,
            &req.scenario_id,
            req.starts.len() as i32,
            valid && cost.is_some(),
            cost,
            makespan,
            solver_result.stats.instruction_count.map(|c| c as i64),
            solver_result.stats.execution_time_ms as i64,
            error_message.as_deref(),
        )
        .await?;

    tracing::info!(
        "Submission {} verified: valid={}, cost={:?}",
        submission.id,
        valid && cost.is_some(),
        cost
    );

    Ok(Json(SubmitResponse {
        submission_id: submission.id.to_string(),
        verification_id: verification.id.to_string(),
        message: if valid && cost.is_some() {
            "Submission verified and added to leaderboard".to_string()
        } else {
            format!(
                "Submission recorded but not valid: {}",
                error_message.unwrap_or_else(|| "Unknown error".to_string())
            )
        },
    }))
}
