use std::time::{SystemTime, UNIX_EPOCH};

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::runtime::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64,
    pub username: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
    pub jti: String,
}

pub fn encode_token(
    secret: &str,
    ttl_hours: u64,
    user_id: u64,
    username: &str,
    role: &str,
) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .as_secs() as usize;
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + (ttl_hours as usize * 3600),
        jti: uuid::Uuid::now_v7().to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(data.claims)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed =
        PasswordHash::new(password_hash).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-with-at-least-thirty-two-bytes";

    #[test]
    fn password_hash_roundtrip() {
        let hash = hash_password("admin123456").expect("hash password");
        assert!(verify_password("admin123456", &hash).expect("verify password"));
        assert!(!verify_password("wrong-password", &hash).expect("verify password"));
    }

    #[test]
    fn jwt_roundtrip() {
        let token = encode_token(SECRET, 1, 7, "admin", "admin").expect("encode token");
        let claims = decode_token(&token, SECRET).expect("decode token");
        assert_eq!(claims.sub, 7);
        assert_eq!(claims.username, "admin");
    }

    #[test]
    fn jwt_rejects_wrong_secret() {
        let token = encode_token(SECRET, 1, 7, "admin", "admin").expect("encode token");
        assert!(decode_token(&token, "another-secret-with-at-least-32-bytes").is_err());
    }
}
