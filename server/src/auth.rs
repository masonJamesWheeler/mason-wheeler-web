use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

use crate::db::get_db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub name: String,
    pub session_id: Option<String>,
    pub exp: usize,
}

static SESSION_SECRET: OnceLock<String> = OnceLock::new();

fn get_secret() -> &'static str {
    SESSION_SECRET.get_or_init(|| {
        std::env::var("SESSION_SECRET")
            .expect("SESSION_SECRET environment variable must be set")
    })
}

pub fn generate_token(
    user_id: &str,
    email: &str,
    role: &str,
    name: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    // Create a session in the database and embed its ID in the token
    let session_id = create_session(user_id);

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        name: name.to_string(),
        session_id: Some(session_id),
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

    let claims = token_data.claims;

    // Check session inactivity (24-hour timeout)
    if let Some(ref session_id) = claims.session_id {
        let db = get_db();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let inactivity_cutoff = (Utc::now() - chrono::Duration::hours(24))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // Check if session exists and was used within the last 24 hours
        let session_valid = db
            .query_row(
                "SELECT 1 FROM sessions WHERE id = ?1 AND last_used > ?2",
                rusqlite::params![session_id, inactivity_cutoff],
                |_| Ok(()),
            )
            .is_ok();

        if !session_valid {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::ExpiredSignature,
            ));
        }

        // Update last_used timestamp
        db.execute(
            "UPDATE sessions SET last_used = ?1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        )
        .ok();
    }

    Ok(claims)
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

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let db = get_db();
    db.execute(
        "INSERT INTO sessions (id, user_id, expires_at, last_used) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![session_id, user_id, expires_at, now],
    )
    .expect("Failed to create session");

    session_id
}

#[allow(dead_code)]
pub fn validate_session(session_id: &str) -> Option<(String, String, String, String)> {
    let db = get_db();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Check session exists, hasn't expired, and was used within the last 24 hours
    let inactivity_cutoff = (Utc::now() - chrono::Duration::hours(24))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let result = db.query_row(
        "SELECT u.id, u.email, u.role, u.name
         FROM sessions s
         JOIN users u ON s.user_id = u.id
         WHERE s.id = ?1 AND s.expires_at > ?2 AND s.last_used > ?3",
        rusqlite::params![session_id, now, inactivity_cutoff],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    )
    .ok();

    // Update last_used timestamp on successful validation
    if result.is_some() {
        db.execute(
            "UPDATE sessions SET last_used = ?1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        )
        .ok();
    }

    result
}

pub fn auth_cookie(token: &str) -> String {
    format!(
        "token={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=2592000",
        token
    )
}
