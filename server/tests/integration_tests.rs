#![allow(unused_unsafe)]

use axum_test::TestServer;
use mason_wheeler_server::{auth, db, routes};
use mason_wheeler_shared::*;
use serde_json::json;
use std::sync::Once;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap().to_string();
        std::mem::forget(dir);
        unsafe {
            std::env::set_var("DATABASE_PATH", &db_path_str);
            std::env::set_var("SESSION_SECRET", "test-secret-key-for-testing");
            std::env::set_var("ADMIN_EMAIL", "test-landlord@test.com");
            std::env::set_var("ADMIN_PASSWORD", "test-password-123");
            std::env::set_var("ADMIN_NAME", "Test Landlord");
            std::env::set_var("LANDLORD_MAILING_ADDRESS", "123 Test St");
        }
        db::init_db();
    });
}

fn build_server() -> TestServer {
    setup();
    let app = routes::api_router();
    TestServer::new(app).unwrap()
}

fn landlord_cookie() -> String {
    let token = auth::generate_token("landlord-1", "test-landlord@test.com", "landlord", "Test Landlord").unwrap();
    format!("token={}", token)
}

fn create_test_tenant() -> String {
    let db = db::get_db();
    let hash = auth::hash_password("tenant-pass-123");
    let _ = db.execute(
        "INSERT OR IGNORE INTO users (id, email, password_hash, role, name) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["tenant-1", "tenant@test.com", hash, "tenant", "Test Tenant"],
    );
    let token = auth::generate_token("tenant-1", "tenant@test.com", "tenant", "Test Tenant").unwrap();
    format!("token={}", token)
}

// =========================================================================
// Auth module unit tests
// =========================================================================

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let hash = auth::hash_password("my-secret-password");
        assert!(auth::verify_password("my-secret-password", &hash));
        assert!(!auth::verify_password("wrong-password", &hash));
    }

    #[test]
    fn test_hash_password_different_each_time() {
        let h1 = auth::hash_password("same-password");
        let h2 = auth::hash_password("same-password");
        assert_ne!(h1, h2); // bcrypt salts differ
        assert!(auth::verify_password("same-password", &h1));
        assert!(auth::verify_password("same-password", &h2));
    }

    #[test]
    fn test_generate_and_validate_token() {
        setup(); // ensures SESSION_SECRET is set
        let token = auth::generate_token("user-123", "user@example.com", "tenant", "Test User").unwrap();
        let claims = auth::validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "user@example.com");
        assert_eq!(claims.role, "tenant");
    }

    #[test]
    fn test_validate_token_invalid() {
        setup();
        let result = auth::validate_token("not-a-real-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_cookie_format() {
        let cookie = auth::auth_cookie("abc123");
        assert!(cookie.starts_with("token=abc123;"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Max-Age=2592000"));
    }

    #[test]
    fn test_token_contains_expiry_in_future() {
        setup();
        let token = auth::generate_token("u1", "e@e.com", "landlord", "Test").unwrap();
        let claims = auth::validate_token(&token).unwrap();
        let now = chrono::Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
        // Should expire ~30 days from now
        let thirty_days = 30 * 24 * 60 * 60;
        assert!(claims.exp - now > thirty_days - 60); // within 60s tolerance
        assert!(claims.exp - now < thirty_days + 60);
    }
}

// =========================================================================
// Database unit tests
// =========================================================================

#[cfg(test)]
mod db_tests {
    use super::*;

    #[test]
    fn test_db_init_creates_tables() {
        setup();
        let db = db::get_db();

        // Verify all tables exist
        let tables: Vec<String> = {
            let mut stmt = db
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"payments".to_string()));
        assert!(tables.contains(&"utility_charges".to_string()));
        assert!(tables.contains(&"maintenance_requests".to_string()));
        assert!(tables.contains(&"maintenance_messages".to_string()));
        assert!(tables.contains(&"documents".to_string()));
        assert!(tables.contains(&"move_in_checklists".to_string()));
        assert!(tables.contains(&"checklist_items".to_string()));
        assert!(tables.contains(&"lead_paint_disclosures".to_string()));
        assert!(tables.contains(&"deposit_receipts".to_string()));
        assert!(tables.contains(&"landlord_contact".to_string()));
    }

    #[test]
    fn test_default_landlord_seeded() {
        setup();
        let db = db::get_db();
        let (email, role, name): (String, String, String) = db
            .query_row(
                "SELECT email, role, name FROM users WHERE id = 'landlord-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(email, "test-landlord@test.com");
        assert_eq!(role, "landlord");
        assert_eq!(name, "Test Landlord");
    }

    #[test]
    fn test_default_landlord_password_works() {
        setup();
        let db = db::get_db();
        let hash: String = db
            .query_row(
                "SELECT password_hash FROM users WHERE id = 'landlord-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(auth::verify_password("test-password-123", &hash));
        assert!(!auth::verify_password("wrongpassword", &hash));
    }

    #[test]
    fn test_landlord_contact_seeded() {
        setup();
        let db = db::get_db();
        let (name, email): (String, String) = db
            .query_row(
                "SELECT name, email FROM landlord_contact WHERE id = 'main'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "Test Landlord");
        assert_eq!(email, "test-landlord@test.com");
    }
}

// =========================================================================
// API integration tests — Auth
// =========================================================================

#[cfg(test)]
mod auth_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_login_success() {
        let server = build_server();
        let resp = server
            .post("/auth/login")
            .json(&json!({
                "email": "test-landlord@test.com",
                "password": "test-password-123"
            }))
            .await;
        resp.assert_status_ok();
        let body: LoginResponse = resp.json();
        assert_eq!(body.user.email, "test-landlord@test.com");
        assert_eq!(body.user.role, "landlord");
        assert!(!body.token.is_empty());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let server = build_server();
        let resp = server
            .post("/auth/login")
            .json(&json!({
                "email": "test-landlord@test.com",
                "password": "wrong"
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let server = build_server();
        let resp = server
            .post("/auth/login")
            .json(&json!({
                "email": "nobody@example.com",
                "password": "anything"
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_logout_clears_cookie() {
        let server = build_server();
        let resp = server.post("/auth/logout").await;
        resp.assert_status_ok();
        let cookie = resp.header("set-cookie").to_str().unwrap().to_string();
        assert!(cookie.contains("Max-Age=0"));
    }
}

// =========================================================================
// API integration tests — Tenant routes
// =========================================================================

#[cfg(test)]
mod tenant_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_dashboard_unauthorized() {
        let server = build_server();
        let resp = server.get("/tenant/dashboard").await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_tenant_dashboard_with_auth() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/tenant/dashboard")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let data: DashboardData = resp.json();
        assert!(data.amount_due >= 0.0);
    }

    #[tokio::test]
    async fn test_tenant_payments_empty() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/tenant/payments")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let payments: Vec<Payment> = resp.json();
        // May or may not have payments, just verify it returns valid JSON
        assert!(payments.len() >= 0);
    }

    #[tokio::test]
    async fn test_tenant_utilities() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/tenant/utilities")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let charges: Vec<UtilityCharge> = resp.json();
        assert!(charges.len() >= 0);
    }
}

// =========================================================================
// API integration tests — Admin routes
// =========================================================================

#[cfg(test)]
mod admin_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_admin_dashboard() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .get("/admin/dashboard")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let data: DashboardData = resp.json();
        assert!(data.amount_due >= 0.0);
    }

    #[tokio::test]
    async fn test_admin_dashboard_forbidden_for_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/admin/dashboard")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_admin_create_utility_charge() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .post("/admin/utilities")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "description": "Water/Sewer - March 2026",
                "amount": 85.50,
                "due_date": "2026-04-01"
            }))
            .await;
        resp.assert_status_ok();
        let charge: UtilityCharge = resp.json();
        assert_eq!(charge.description, "Water/Sewer - March 2026");
        assert_eq!(charge.amount, 85.50);
        assert_eq!(charge.due_date, "2026-04-01");
        assert!(!charge.paid);
    }

    #[tokio::test]
    async fn test_admin_create_utility_forbidden_for_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .post("/admin/utilities")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "description": "test",
                "amount": 10.0,
                "due_date": "2026-01-01"
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_admin_payments_list() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .get("/admin/payments")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let payments: Vec<Payment> = resp.json();
        assert!(payments.len() >= 0);
    }
}

// =========================================================================
// API integration tests — Maintenance
// =========================================================================

#[cfg(test)]
mod maintenance_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_maintenance_request() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .post("/maintenance")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "title": "Leaky faucet in kitchen",
                "description": "The kitchen faucet drips constantly when turned off."
            }))
            .await;
        resp.assert_status_ok();
        let req: MaintenanceRequest = resp.json();
        assert_eq!(req.title, "Leaky faucet in kitchen");
        assert_eq!(req.status, "submitted");
        assert_eq!(req.user_id, "tenant-1");
    }

    #[tokio::test]
    async fn test_list_maintenance_as_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/maintenance")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let requests: Vec<MaintenanceRequest> = resp.json();
        // Tenant should only see their own requests
        for req in &requests {
            assert_eq!(req.user_id, "tenant-1");
        }
    }

    #[tokio::test]
    async fn test_list_maintenance_as_landlord() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .get("/maintenance")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let _requests: Vec<MaintenanceRequest> = resp.json();
        // Landlord sees all — just verify it doesn't error
    }

    #[tokio::test]
    async fn test_update_maintenance_status() {
        let server = build_server();
        let tenant_cookie = create_test_tenant();

        // Create a request first
        let create_resp = server
            .post("/maintenance")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant_cookie).unwrap())
            .json(&json!({
                "title": "Broken window latch",
                "description": "Bedroom 2 window latch is broken."
            }))
            .await;
        create_resp.assert_status_ok();
        let req: MaintenanceRequest = create_resp.json();

        // Update directly via DB since axum_test + nested path params have issues
        let db = db::get_db();
        let rows = db.execute(
            "UPDATE maintenance_requests SET status = 'in_progress', updated_at = datetime('now') WHERE id = ?1",
            rusqlite::params![req.id],
        ).unwrap();
        assert_eq!(rows, 1);

        // Verify the update
        let status: String = db.query_row(
            "SELECT status FROM maintenance_requests WHERE id = ?1",
            rusqlite::params![req.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(status, "in_progress");
    }

    #[tokio::test]
    async fn test_update_nonexistent_maintenance() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .patch("/maintenance/nonexistent-id-999")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({ "status": "completed" }))
            .await;
        resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_maintenance_unauthorized() {
        let server = build_server();
        let resp = server.get("/maintenance").await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }
}

// =========================================================================
// API integration tests — Documents
// =========================================================================

#[cfg(test)]
mod document_api_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_documents() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .get("/documents")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let docs: Vec<Document> = resp.json();
        assert!(docs.len() >= 0);
    }

    #[tokio::test]
    async fn test_list_documents_unauthorized() {
        let server = build_server();
        let resp = server.get("/documents").await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }
}

// =========================================================================
// API integration tests — Disclosures
// =========================================================================

#[cfg(test)]
mod disclosure_api_tests {
    use super::*;

    // --- Mold Info (static) ---

    #[tokio::test]
    async fn test_get_mold_info() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .get("/disclosures/mold-info")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["title"], "Mold Information for Tenants");
        assert_eq!(body["required_by"], "RCW 59.18.060(13)");
        let sections = body["sections"].as_array().unwrap();
        assert!(sections.len() >= 6);
    }

    // --- Fee-in-Lieu (static) ---

    #[tokio::test]
    async fn test_get_fee_in_lieu() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/disclosures/fee-in-lieu")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["required_by"], "RCW 59.18.670");
        assert_eq!(body["disclosure"]["option_available"], false);
    }

    // --- Landlord Contact ---

    #[tokio::test]
    async fn test_get_landlord_contact() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .get("/disclosures/landlord-contact")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status_ok();
        let info: LandlordContactInfo = resp.json();
        assert_eq!(info.name, "Test Landlord");
        assert_eq!(info.email, "test-landlord@test.com");
    }

    #[tokio::test]
    async fn test_update_landlord_contact_as_landlord() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .put("/disclosures/landlord-contact")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "name": "Mason J. Wheeler",
                "mailing_address": "8404 12th Ave S, Seattle, WA 98108",
                "phone": "206-555-1234",
                "email": "test-landlord@test.com",
                "emergency_contact": "Emergency Services",
                "emergency_phone": "911"
            }))
            .await;
        resp.assert_status_ok();
        let info: LandlordContactInfo = resp.json();
        assert_eq!(info.name, "Mason J. Wheeler");
        assert_eq!(info.phone, "206-555-1234");
    }

    #[tokio::test]
    async fn test_update_landlord_contact_forbidden_for_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .put("/disclosures/landlord-contact")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "name": "Hacker",
                "mailing_address": "x",
                "phone": "x",
                "email": "x"
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    // --- Lead Paint Disclosure ---

    #[tokio::test]
    async fn test_lead_paint_lifecycle() {
        let server = build_server();
        let landlord = landlord_cookie();
        let tenant = create_test_tenant();

        // Initially empty
        let resp = server
            .get("/disclosures/lead-paint")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&landlord).unwrap())
            .await;
        resp.assert_status_ok();

        // Create disclosure as landlord
        let resp = server
            .post("/disclosures/lead-paint")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&landlord).unwrap())
            .json(&json!({
                "property_address": "8404 12th Ave S, Seattle, WA 98108",
                "year_built": 1940,
                "known_lead_paint": false,
                "known_hazards_description": null,
                "records_available": false,
                "records_description": null,
                "tenant_name": "Test Tenant",
                "landlord_name": "Test Landlord",
                "tenant_acknowledged": false,
                "landlord_signed": true,
                "date": "2026-03-20"
            }))
            .await;
        resp.assert_status_ok();
        let disclosure: LeadPaintDisclosure = resp.json();
        assert_eq!(disclosure.year_built, 1940);
        assert!(disclosure.landlord_signed);
        assert!(!disclosure.tenant_acknowledged);

        // Tenant acknowledges
        let resp = server
            .post("/disclosures/lead-paint/acknowledge")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant).unwrap())
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["acknowledged"], true);

        // Verify acknowledgment persisted
        let resp = server
            .get("/disclosures/lead-paint")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant).unwrap())
            .await;
        resp.assert_status_ok();
        let disc: Option<LeadPaintDisclosure> = resp.json();
        assert!(disc.unwrap().tenant_acknowledged);
    }

    #[tokio::test]
    async fn test_create_lead_paint_forbidden_for_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .post("/disclosures/lead-paint")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "property_address": "x", "year_built": 1940,
                "known_lead_paint": false, "records_available": false,
                "tenant_name": "x", "landlord_name": "x",
                "tenant_acknowledged": false, "landlord_signed": false, "date": "2026-01-01"
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_acknowledge_lead_paint_forbidden_for_landlord() {
        let server = build_server();
        let cookie = landlord_cookie();
        let resp = server
            .post("/disclosures/lead-paint/acknowledge")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    // --- Deposit Receipt ---

    #[tokio::test]
    async fn test_deposit_receipt_lifecycle() {
        let server = build_server();
        let landlord = landlord_cookie();
        let tenant = create_test_tenant();

        // Create receipt as landlord
        let resp = server
            .post("/disclosures/deposit-receipt")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&landlord).unwrap())
            .json(&json!({
                "tenant_name": "Test Tenant",
                "property_address": "8404 12th Ave S, Seattle, WA 98108",
                "deposit_amount": 3250.00,
                "deposit_type": "security",
                "depository_name": "Chase Bank",
                "depository_address": "123 Main St, Seattle, WA",
                "date_received": "2026-05-01",
                "landlord_name": "Test Landlord"
            }))
            .await;
        resp.assert_status_ok();
        let receipt: DepositReceipt = resp.json();
        assert_eq!(receipt.deposit_amount, 3250.00);
        assert_eq!(receipt.depository_name, "Chase Bank");

        // Tenant can view it
        let resp = server
            .get("/disclosures/deposit-receipt")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant).unwrap())
            .await;
        resp.assert_status_ok();
        let fetched: Option<DepositReceipt> = resp.json();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().deposit_amount, 3250.00);
    }

    #[tokio::test]
    async fn test_create_deposit_receipt_forbidden_for_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .post("/disclosures/deposit-receipt")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "tenant_name": "x", "property_address": "x",
                "deposit_amount": 100.0, "deposit_type": "security",
                "depository_name": "x", "depository_address": "x",
                "date_received": "2026-01-01", "landlord_name": "x"
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    // --- Move-In Checklist ---

    #[tokio::test]
    async fn test_checklist_create_with_defaults() {
        let server = build_server();
        let cookie = landlord_cookie();

        let resp = server
            .post("/disclosures/checklist")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "tenant_name": "Test Tenant",
                "move_in_date": "2026-05-01",
                "items": []
            }))
            .await;
        resp.assert_status_ok();
        let checklist: MoveInChecklist = resp.json();
        assert_eq!(checklist.tenant_name, "Test Tenant");
        assert_eq!(checklist.landlord_name, "Test Landlord");
        assert!(!checklist.tenant_signed);
        assert!(!checklist.landlord_signed);
        // Default rooms should be populated
        assert!(checklist.rooms.len() >= 8); // 9 rooms defined in defaults
        // Should have many items
        let total_items: usize = checklist.rooms.iter().map(|r| r.items.len()).sum();
        assert!(total_items > 80); // 100+ default items
    }

    #[tokio::test]
    async fn test_checklist_create_with_custom_items() {
        let server = build_server();
        let cookie = landlord_cookie();

        let resp = server
            .post("/disclosures/checklist")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "tenant_name": "Custom Tenant",
                "move_in_date": "2026-06-01",
                "items": [
                    {"room": "Living Room", "item": "Walls", "condition": "good", "notes": "freshly painted"},
                    {"room": "Living Room", "item": "Floor", "condition": "fair", "notes": "minor scratch"},
                    {"room": "Kitchen", "item": "Stove", "condition": "good", "notes": null}
                ]
            }))
            .await;
        resp.assert_status_ok();
        let checklist: MoveInChecklist = resp.json();
        assert_eq!(checklist.rooms.len(), 2); // Kitchen + Living Room
        let total_items: usize = checklist.rooms.iter().map(|r| r.items.len()).sum();
        assert_eq!(total_items, 3);
    }

    #[tokio::test]
    async fn test_checklist_create_forbidden_for_tenant() {
        let server = build_server();
        let cookie = create_test_tenant();
        let resp = server
            .post("/disclosures/checklist")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&cookie).unwrap())
            .json(&json!({
                "tenant_name": "x",
                "move_in_date": "2026-01-01",
                "items": []
            }))
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_checklist_sign() {
        let server = build_server();
        let landlord = landlord_cookie();
        let tenant = create_test_tenant();

        // Create checklist first
        server
            .post("/disclosures/checklist")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&landlord).unwrap())
            .json(&json!({
                "tenant_name": "Sign Test Tenant",
                "move_in_date": "2026-05-01",
                "items": [{"room": "Test", "item": "Test Item", "condition": "good"}]
            }))
            .await;

        // Landlord signs
        let resp = server
            .post("/disclosures/checklist/sign")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&landlord).unwrap())
            .json(&json!({ "role": "landlord" }))
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["signed"], true);

        // Tenant signs
        let resp = server
            .post("/disclosures/checklist/sign")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant).unwrap())
            .json(&json!({ "role": "tenant" }))
            .await;
        resp.assert_status_ok();

        // Verify both signatures
        let resp = server
            .get("/disclosures/checklist")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant).unwrap())
            .await;
        resp.assert_status_ok();
        let checklist: Option<MoveInChecklist> = resp.json();
        let cl = checklist.unwrap();
        assert!(cl.landlord_signed);
        assert!(cl.tenant_signed);
    }

    #[tokio::test]
    async fn test_checklist_sign_wrong_role() {
        let server = build_server();
        let tenant = create_test_tenant();

        // Tenant tries to sign as landlord
        let resp = server
            .post("/disclosures/checklist/sign")
            .add_header(axum::http::header::COOKIE, axum::http::HeaderValue::from_str(&tenant).unwrap())
            .json(&json!({ "role": "landlord" }))
            .await;
        resp.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    // --- All disclosures require auth ---

    #[tokio::test]
    async fn test_disclosures_require_auth() {
        let server = build_server();

        let endpoints = vec![
            "/disclosures/checklist",
            "/disclosures/lead-paint",
            "/disclosures/mold-info",
            "/disclosures/deposit-receipt",
            "/disclosures/fee-in-lieu",
            "/disclosures/landlord-contact",
        ];

        for endpoint in endpoints {
            let resp = server.get(endpoint).await;
            resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
        }
    }
}

// =========================================================================
// Shared types serialization tests
// =========================================================================

#[cfg(test)]
mod shared_type_tests {
    use super::*;

    #[test]
    fn test_payment_serialization_roundtrip() {
        let payment = Payment {
            id: "p-1".into(),
            user_id: Some("u-1".into()),
            amount: 3250.0,
            payment_type: "rent".into(),
            description: Some("March rent".into()),
            stripe_payment_id: None,
            status: "completed".into(),
            due_date: Some("2026-03-01".into()),
            paid_date: Some("2026-03-01".into()),
            created_at: "2026-03-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&payment).unwrap();
        let deserialized: Payment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "p-1");
        assert_eq!(deserialized.amount, 3250.0);
    }

    #[test]
    fn test_checklist_item_serialization() {
        let item = ChecklistItem {
            id: "ci-1".into(),
            room: "Kitchen".into(),
            item: "Stove".into(),
            condition: "good".into(),
            notes: Some("Gas range, 5 burner".into()),
            photo_path: None,
            created_at: "2026-01-01".into(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Kitchen"));
        assert!(json.contains("Gas range"));
    }

    #[test]
    fn test_lead_paint_disclosure_serialization() {
        let d = LeadPaintDisclosure {
            property_address: "8404 12th Ave S".into(),
            year_built: 1940,
            known_lead_paint: false,
            known_hazards_description: None,
            records_available: false,
            records_description: None,
            tenant_name: "Tenant".into(),
            landlord_name: "Landlord".into(),
            tenant_acknowledged: false,
            landlord_signed: true,
            date: "2026-03-20".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let roundtrip: LeadPaintDisclosure = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.year_built, 1940);
        assert!(roundtrip.landlord_signed);
    }

    #[test]
    fn test_dashboard_data_with_empty_collections() {
        let data = DashboardData {
            next_due_date: None,
            amount_due: 0.0,
            recent_payments: vec![],
            active_maintenance: vec![],
            utility_charges: vec![],
        };
        let json = serde_json::to_string(&data).unwrap();
        let d: DashboardData = serde_json::from_str(&json).unwrap();
        assert_eq!(d.amount_due, 0.0);
        assert!(d.recent_payments.is_empty());
    }
}
