use axum::http::HeaderMap;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::store::DataStore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub display_name: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: Claims,
}

pub async fn register(store: &DataStore, payload: AuthPayload) -> anyhow::Result<AuthResponse> {
    let user_id = Uuid::new_v4();
    let display_name = payload
        .display_name
        .clone()
        .unwrap_or_else(|| payload.email.split('@').next().unwrap_or("operator").to_string());
    sqlx::query("INSERT INTO users (id, email, password_hash, display_name) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(&payload.email)
        .bind(hash_password(&payload.password))
        .bind(&display_name)
        .execute(&store.db)
        .await?;
    issue_token(store, user_id.to_string(), payload.email, display_name)
}

pub async fn login(store: &DataStore, payload: AuthPayload) -> anyhow::Result<AuthResponse> {
    let row = sqlx::query("SELECT id, email, password_hash, display_name FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&store.db)
        .await?;
    let password_hash: String = row.get("password_hash");
    if password_hash != hash_password(&payload.password) {
        anyhow::bail!("invalid credentials");
    }
    issue_token(
        store,
        row.get::<Uuid, _>("id").to_string(),
        row.get("email"),
        row.get("display_name"),
    )
}

pub fn auth_from_headers(store: &DataStore, headers: &HeaderMap) -> Option<Claims> {
    let header = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(store.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?;
    Some(decoded.claims)
}

fn issue_token(store: &DataStore, sub: String, email: String, display_name: String) -> anyhow::Result<AuthResponse> {
    let claims = Claims {
        sub,
        email,
        display_name,
        exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(store.config.jwt_secret.as_bytes()),
    )?;
    Ok(AuthResponse { token, user: claims })
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    STANDARD_NO_PAD.encode(hasher.finalize())
}
