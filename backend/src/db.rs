use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// Database models

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SolverSubmission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub solver_name: String,
    pub wasm_hash: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct VerificationResult {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub map_name: String,
    pub scenario_id: String,
    pub num_agents: i32,
    pub valid: bool,
    pub cost: Option<i64>,
    pub makespan: Option<i64>,
    pub instruction_count: Option<i64>,
    pub execution_time_ms: i64,
    pub error_message: Option<String>,
    pub verified_at: DateTime<Utc>,
}

// Repository functions

impl Database {
    // User operations
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *",
        )
        .bind(username)
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
    }

    // API key operations
    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        key_hash: &str,
        name: &str,
    ) -> Result<ApiKey, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "INSERT INTO api_keys (user_id, key_hash, name) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(user_id)
        .bind(key_hash)
        .bind(name)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND revoked = false",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_api_key_last_used(&self, key_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Solver submission operations
    pub async fn create_submission(
        &self,
        user_id: Uuid,
        solver_name: &str,
        wasm_hash: &str,
    ) -> Result<SolverSubmission, sqlx::Error> {
        sqlx::query_as::<_, SolverSubmission>(
            "INSERT INTO solver_submissions (user_id, solver_name, wasm_hash) 
             VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(user_id)
        .bind(solver_name)
        .bind(wasm_hash)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_submission(&self, id: Uuid) -> Result<Option<SolverSubmission>, sqlx::Error> {
        sqlx::query_as::<_, SolverSubmission>("SELECT * FROM solver_submissions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    // Verification result operations
    pub async fn create_verification_result(
        &self,
        submission_id: Uuid,
        map_name: &str,
        scenario_id: &str,
        num_agents: i32,
        valid: bool,
        cost: Option<i64>,
        makespan: Option<i64>,
        instruction_count: Option<i64>,
        execution_time_ms: i64,
        error_message: Option<&str>,
    ) -> Result<VerificationResult, sqlx::Error> {
        sqlx::query_as::<_, VerificationResult>(
            "INSERT INTO verification_results 
             (submission_id, map_name, scenario_id, num_agents, valid, cost, makespan, 
              instruction_count, execution_time_ms, error_message)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
        )
        .bind(submission_id)
        .bind(map_name)
        .bind(scenario_id)
        .bind(num_agents)
        .bind(valid)
        .bind(cost)
        .bind(makespan)
        .bind(instruction_count)
        .bind(execution_time_ms)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_leaderboard(
        &self,
        map_name: Option<&str>,
        limit: i64,
    ) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let query = if let Some(map) = map_name {
            sqlx::query_as::<_, LeaderboardEntry>(
                "SELECT 
                    u.username,
                    ss.solver_name,
                    vr.map_name,
                    vr.scenario_id,
                    vr.num_agents,
                    vr.cost,
                    vr.makespan,
                    vr.instruction_count,
                    vr.execution_time_ms,
                    vr.verified_at
                FROM verification_results vr
                JOIN solver_submissions ss ON vr.submission_id = ss.id
                JOIN users u ON ss.user_id = u.id
                WHERE vr.valid = true AND vr.map_name = $1
                ORDER BY vr.cost ASC, vr.instruction_count ASC
                LIMIT $2",
            )
            .bind(map)
            .bind(limit)
        } else {
            sqlx::query_as::<_, LeaderboardEntry>(
                "SELECT 
                    u.username,
                    ss.solver_name,
                    vr.map_name,
                    vr.scenario_id,
                    vr.num_agents,
                    vr.cost,
                    vr.makespan,
                    vr.instruction_count,
                    vr.execution_time_ms,
                    vr.verified_at
                FROM verification_results vr
                JOIN solver_submissions ss ON vr.submission_id = ss.id
                JOIN users u ON ss.user_id = u.id
                WHERE vr.valid = true
                ORDER BY vr.cost ASC, vr.instruction_count ASC
                LIMIT $1",
            )
            .bind(limit)
        };

        query.fetch_all(&self.pool).await
    }
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct LeaderboardEntry {
    pub username: String,
    pub solver_name: String,
    pub map_name: String,
    pub scenario_id: String,
    pub num_agents: i32,
    pub cost: Option<i64>,
    pub makespan: Option<i64>,
    pub instruction_count: Option<i64>,
    pub execution_time_ms: i64,
    pub verified_at: DateTime<Utc>,
}
