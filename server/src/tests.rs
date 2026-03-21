use std::sync::Mutex;

use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use rusqlite::Connection;

use crate::auth;

// ---------------------------------------------------------------------------
// Helper: create an in-memory SQLite database with the app schema and seed
// a landlord user so route handlers that require auth can work.
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory db");
    conn.execute_batch("PRAGMA foreign_keys=ON;").ok();

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('landlord', 'tenant')),
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS payments (
            id TEXT PRIMARY KEY,
            user_id TEXT REFERENCES users(id),
            amount REAL NOT NULL,
            payment_type TEXT NOT NULL,
            description TEXT,
            stripe_payment_id TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            due_date TEXT,
            paid_date TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS utility_charges (
            id TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            due_date TEXT NOT NULL,
            paid INTEGER NOT NULL DEFAULT 0,
            payment_id TEXT REFERENCES payments(id),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS maintenance_requests (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'submitted',
            photo_path TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS maintenance_messages (
            id TEXT PRIMARY KEY,
            request_id TEXT NOT NULL REFERENCES maintenance_requests(id),
            user_id TEXT NOT NULL REFERENCES users(id),
            message TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            doc_type TEXT NOT NULL,
            file_path TEXT NOT NULL,
            uploaded_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS move_in_checklists (
            id TEXT PRIMARY KEY,
            property_address TEXT NOT NULL,
            tenant_name TEXT NOT NULL,
            landlord_name TEXT NOT NULL,
            move_in_date TEXT NOT NULL,
            tenant_signed INTEGER NOT NULL DEFAULT 0,
            landlord_signed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS checklist_items (
            id TEXT PRIMARY KEY,
            checklist_id TEXT NOT NULL REFERENCES move_in_checklists(id),
            room TEXT NOT NULL,
            item TEXT NOT NULL,
            condition TEXT NOT NULL DEFAULT 'good',
            notes TEXT,
            photo_path TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS lead_paint_disclosures (
            id TEXT PRIMARY KEY,
            property_address TEXT NOT NULL,
            year_built INTEGER NOT NULL,
            known_lead_paint INTEGER NOT NULL DEFAULT 0,
            known_hazards_description TEXT,
            records_available INTEGER NOT NULL DEFAULT 0,
            records_description TEXT,
            tenant_name TEXT NOT NULL,
            landlord_name TEXT NOT NULL,
            tenant_acknowledged INTEGER NOT NULL DEFAULT 0,
            landlord_signed INTEGER NOT NULL DEFAULT 0,
            date_signed TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS deposit_receipts (
            id TEXT PRIMARY KEY,
            tenant_name TEXT NOT NULL,
            property_address TEXT NOT NULL,
            deposit_amount REAL NOT NULL,
            deposit_type TEXT NOT NULL DEFAULT 'security',
            depository_name TEXT NOT NULL,
            depository_address TEXT NOT NULL,
            date_received TEXT NOT NULL,
            landlord_name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS signatures (
            id TEXT PRIMARY KEY,
            document_type TEXT NOT NULL,
            document_id TEXT NOT NULL,
            signer_role TEXT NOT NULL,
            signer_name TEXT NOT NULL,
            signature_path TEXT NOT NULL,
            signed_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS landlord_contact (
            id TEXT PRIMARY KEY DEFAULT 'main',
            name TEXT NOT NULL,
            mailing_address TEXT NOT NULL,
            phone TEXT NOT NULL,
            email TEXT NOT NULL,
            emergency_contact TEXT,
            emergency_phone TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS password_reset_tokens (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            token TEXT NOT NULL UNIQUE,
            expires_at TEXT NOT NULL,
            used INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        );
        ",
    )
    .expect("Failed to create tables");

    conn
}

/// Seed a landlord user into the given connection and return (id, email, password).
#[allow(dead_code)]
fn seed_landlord(conn: &Connection) -> (String, String, String) {
    let email = "landlord@test.com";
    let password = "testpassword123";
    let hash = auth::hash_password(password);
    conn.execute(
        "INSERT INTO users (id, email, password_hash, role, name) VALUES (?1, ?2, ?3, 'landlord', 'Test Landlord')",
        rusqlite::params!["landlord-1", email, hash],
    )
    .expect("seed landlord");
    ("landlord-1".into(), email.into(), password.into())
}

/// Seed a tenant user into the given connection and return (id, email, password).
#[allow(dead_code)]
fn seed_tenant(conn: &Connection) -> (String, String, String) {
    let email = "tenant@test.com";
    let password = "tenantpass123";
    let hash = auth::hash_password(password);
    conn.execute(
        "INSERT INTO users (id, email, password_hash, role, name) VALUES (?1, ?2, ?3, 'tenant', 'Test Tenant')",
        rusqlite::params!["tenant-1", email, hash],
    )
    .expect("seed tenant");
    ("tenant-1".into(), email.into(), password.into())
}

/// Generate a token cookie string for use in integration tests.
fn auth_cookie_for(user_id: &str, email: &str, role: &str) -> String {
    let token = auth::generate_token(user_id, email, role, "Test User").expect("generate token");
    format!("token={token}")
}

// ===========================================================================
// Auth tests
// ===========================================================================

#[test]
fn test_hash_and_verify_password() {
    let password = "my_secure_password";
    let hash = auth::hash_password(password);

    // Correct password verifies
    assert!(
        auth::verify_password(password, &hash),
        "correct password should verify"
    );

    // Wrong password fails
    assert!(
        !auth::verify_password("wrong_password", &hash),
        "wrong password should not verify"
    );
}

#[test]
fn test_generate_and_validate_token() {
    // generate_token now creates a session row, so we need the DB initialized
    // and the user to exist (foreign key constraint on sessions.user_id).
    ensure_init_db();

    let user_id = "landlord-1"; // seeded by ensure_init_db
    let email = "admin@test.com";
    let role = "landlord";

    let token = auth::generate_token(user_id, email, role, "Test Admin").expect("token generation should succeed");

    let claims = auth::validate_token(&token).expect("token validation should succeed");
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
    assert_eq!(claims.role, role);
    // Expiry should be in the future
    let now = Utc::now().timestamp() as usize;
    assert!(claims.exp > now, "token expiry should be in the future");
}

#[test]
fn test_expired_token_rejected() {
    ensure_init_db();
    let secret =
        std::env::var("SESSION_SECRET").expect("SESSION_SECRET should be set by ensure_init_db");

    let claims = auth::Claims {
        sub: "user-1".to_string(),
        email: "expired@example.com".to_string(),
        role: "tenant".to_string(),
        name: "Expired User".to_string(),
        session_id: None,
        exp: 1_000_000, // far in the past (1970)
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("encoding should succeed");

    let result = auth::validate_token(&token);
    assert!(result.is_err(), "expired token should be rejected");
}

// ===========================================================================
// DB tests  (use a temporary on-disk database via tempfile so init_db works)
// ===========================================================================

/// Serialize DB tests that call `init_db` because it sets a global OnceLock.
/// We only call it once via `std::sync::Once`.
static INIT: std::sync::Once = std::sync::Once::new();
static TEST_DB_PATH: Mutex<Option<tempfile::TempDir>> = Mutex::new(None);

/// Initialize the global database exactly once for integration tests.
fn ensure_init_db() {
    INIT.call_once(|| {
        let tmp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = tmp_dir.path().join("test.db");
        // SAFETY: We call this during test init before any threads are spawned,
        // and test parallelism is managed by the test harness.
        // SAFETY: We call this during test init before any threads are spawned,
        // and test parallelism is managed by the test harness.
        unsafe {
            std::env::set_var("DATABASE_PATH", db_path.to_str().unwrap());
            std::env::set_var("ADMIN_EMAIL", "admin@test.com");
            std::env::set_var("ADMIN_PASSWORD", "adminpass123");
            std::env::set_var("ADMIN_NAME", "Test Admin");
            std::env::set_var("SESSION_SECRET", "test-secret-for-jwt-signing");
        }

        // init_db also creates data/documents and data/signatures directories
        crate::db::init_db();

        // Keep the TempDir alive so the file isn't deleted
        *TEST_DB_PATH.lock().unwrap() = Some(tmp_dir);
    });
}

#[test]
fn test_init_db_creates_tables() {
    ensure_init_db();

    let db = crate::db::get_db();
    let tables: Vec<String> = {
        let mut stmt = db
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    let expected = [
        "checklist_items",
        "deposit_receipts",
        "documents",
        "landlord_contact",
        "lead_paint_disclosures",
        "maintenance_messages",
        "maintenance_requests",
        "move_in_checklists",
        "password_reset_tokens",
        "payments",
        "sessions",
        "signatures",
        "users",
        "utility_charges",
    ];

    for table in &expected {
        assert!(
            tables.contains(&table.to_string()),
            "table '{}' should exist, found: {:?}",
            table,
            tables
        );
    }
}

#[test]
fn test_default_landlord_created() {
    ensure_init_db();

    let db = crate::db::get_db();
    let (email, role): (String, String) = db
        .query_row(
            "SELECT email, role FROM users WHERE id = 'landlord-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("landlord user should exist");

    assert_eq!(email, "admin@test.com");
    assert_eq!(role, "landlord");
}

// ===========================================================================
// Route handler integration tests  (use axum_test + the real global DB)
// ===========================================================================

use axum_test::TestServer;

fn build_test_server() -> TestServer {
    ensure_init_db();
    let app = crate::routes::api_router();
    TestServer::new(app).expect("build test server")
}

#[tokio::test]
async fn test_login_success() {
    let server = build_test_server();

    let resp = server
        .post("/auth/login")
        .json(&serde_json::json!({
            "email": "admin@test.com",
            "password": "adminpass123"
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body.get("token").is_some(), "response should contain token");
    assert_eq!(body["user"]["email"], "admin@test.com");
    assert_eq!(body["user"]["role"], "landlord");
}

#[tokio::test]
async fn test_login_wrong_password() {
    let server = build_test_server();

    let resp = server
        .post("/auth/login")
        .json(&serde_json::json!({
            "email": "admin@test.com",
            "password": "wrongpassword"
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = resp.json();
    assert!(body.get("error").is_some(), "response should contain error");
}

#[tokio::test]
async fn test_login_missing_email() {
    let server = build_test_server();

    let resp = server
        .post("/auth/login")
        .json(&serde_json::json!({
            "email": "",
            "password": "somepassword"
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["field"], "email");
}

#[tokio::test]
async fn test_create_tenant() {
    let server = build_test_server();

    // Get auth cookie for the landlord
    let cookie = auth_cookie_for("landlord-1", "admin@test.com", "landlord");

    let resp = server
        .post("/admin/tenants")
        .add_header(axum::http::header::COOKIE, cookie)
        .json(&serde_json::json!({
            "email": "newtenant@test.com",
            "password": "password1234",
            "name": "New Tenant"
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["email"], "newtenant@test.com");
    assert_eq!(body["name"], "New Tenant");
    assert!(body.get("id").is_some(), "response should contain tenant id");
}

#[tokio::test]
async fn test_create_maintenance_request() {
    let server = build_test_server();

    // First ensure a tenant exists
    {
        let db = crate::db::get_db();
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM users WHERE email = 'maint_tenant@test.com'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if count == 0 {
            let hash = auth::hash_password("tenantpass123");
            db.execute(
                "INSERT INTO users (id, email, password_hash, role, name) VALUES (?1, ?2, ?3, 'tenant', 'Maint Tenant')",
                rusqlite::params!["tenant-maint", "maint_tenant@test.com", hash],
            )
            .expect("insert test tenant for maintenance");
        }
    }

    let cookie = auth_cookie_for("tenant-maint", "maint_tenant@test.com", "tenant");

    let resp = server
        .post("/maintenance")
        .add_header(axum::http::header::COOKIE, cookie)
        .json(&serde_json::json!({
            "title": "Leaky faucet",
            "description": "The kitchen faucet is dripping constantly."
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "Leaky faucet");
    assert_eq!(body["status"], "submitted");
    assert_eq!(body["user_id"], "tenant-maint");
}

#[tokio::test]
async fn test_create_utility_charge() {
    let server = build_test_server();

    let cookie = auth_cookie_for("landlord-1", "admin@test.com", "landlord");

    let resp = server
        .post("/admin/utilities")
        .add_header(axum::http::header::COOKIE, cookie)
        .json(&serde_json::json!({
            "description": "Water bill - March",
            "amount": 75.50,
            "due_date": "2026-04-01"
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["description"], "Water bill - March");
    assert_eq!(body["amount"], 75.5);
    assert_eq!(body["due_date"], "2026-04-01");
    assert_eq!(body["paid"], false);
}
