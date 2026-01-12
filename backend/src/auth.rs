use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use rand::Rng;
use uuid::Uuid;

use crate::{db::Database, error::{AppError, Result}};

/// Generate a new API key (random 32-character string)
pub fn generate_api_key() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const KEY_LEN: usize = 32;
    let mut rng = rand::thread_rng();

    (0..KEY_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Hash an API key using Argon2
pub fn hash_api_key(key: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(key.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash API key: {}", e)))?;
    Ok(password_hash.to_string())
}

/// Verify an API key against a hash
pub fn verify_api_key(key: &str, hash: &str) -> Result<bool> {
    use argon2::PasswordVerifier;
    use argon2::password_hash::PasswordHash;

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash format: {}", e)))?;
    
    Ok(Argon2::default()
        .verify_password(key.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Authenticated user extracted from request
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub api_key_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        // Extract API key from Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Auth("Missing Authorization header".to_string()))?;

        // Parse Bearer token
        let api_key = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Auth("Invalid Authorization header format".to_string()))?;

        // Get database from extensions (added by middleware)
        let db = parts
            .extensions
            .get::<Database>()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Database not in extensions")))?;

        // Hash the provided key and look it up
        // Note: In production, consider using a constant-time comparison
        let key_hash = hash_api_key(api_key)?;
        
        let api_key_record = db
            .get_api_key_by_hash(&key_hash)
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::Auth("Invalid API key".to_string()))?;

        // Update last used timestamp
        db.update_api_key_last_used(api_key_record.id)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(AuthenticatedUser {
            user_id: api_key_record.user_id,
            api_key_id: api_key_record.id,
        })
    }
}
