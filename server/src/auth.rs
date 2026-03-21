use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::get_db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

fn get_secret() -> String {
    std::env::var("SESSION_SECRET").unwrap_or_else(|_| "default-secret-change-me".to_string())
}

pub fn generate_token(
    user_id: &str,
    email: &str,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    let secret = get_secret();
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = get_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> String {
    bcrypt::hash(password, 10).expect("Failed to hash password")
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

pub fn create_session(user_id: &str) -> String {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let db = get_db();
    db.execute(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![session_id, user_id, expires_at],
    )
    .expect("Failed to create session");

    session_id
}

pub fn validate_session(session_id: &str) -> Option<(String, String, String, String)> {
    let db = get_db();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    db.query_row(
        "SELECT u.id, u.email, u.role, u.name
         FROM sessions s
         JOIN users u ON s.user_id = u.id
         WHERE s.id = ?1 AND s.expires_at > ?2",
        rusqlite::params![session_id, now],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    )
    .ok()
}

pub fn auth_cookie(token: &str) -> String {
    format!(
        "token={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=2592000",
        token
    )
}
