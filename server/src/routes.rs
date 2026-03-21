use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query},
    http::{header, header::SET_COOKIE, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use mason_wheeler_shared::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use uuid::Uuid;

use crate::auth;
use crate::db::get_db;
use crate::email;
use crate::pdf;

// ---------------------------------------------------------------------------
// Server start time (for uptime tracking)
// ---------------------------------------------------------------------------

static START_TIME: OnceLock<Instant> = OnceLock::new();

fn server_start_time() -> &'static Instant {
    START_TIME.get_or_init(Instant::now)
}

// ---------------------------------------------------------------------------
// Rate limiting for login attempts
// ---------------------------------------------------------------------------

static LOGIN_ATTEMPTS: OnceLock<Mutex<HashMap<String, (u32, Instant)>>> = OnceLock::new();

fn get_login_attempts() -> &'static Mutex<HashMap<String, (u32, Instant)>> {
    LOGIN_ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}

const MAX_LOGIN_ATTEMPTS: u32 = 5;
const LOGIN_WINDOW_SECS: u64 = 15 * 60; // 15 minutes

const MAX_RATE_LIMIT_ENTRIES: usize = 1000;

fn check_rate_limit(key: &str) -> Result<(), StatusCode> {
    let mut attempts = get_login_attempts().lock().unwrap();

    // Periodic cleanup: remove entries older than the login window
    attempts.retain(|_, (_, first_attempt)| first_attempt.elapsed().as_secs() < LOGIN_WINDOW_SECS);

    // Hard cap: if still over the limit, remove the oldest entries
    if attempts.len() > MAX_RATE_LIMIT_ENTRIES {
        let mut entries: Vec<(String, Instant)> = attempts
            .iter()
            .map(|(k, (_, t))| (k.clone(), *t))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1)); // newest first
        entries.truncate(MAX_RATE_LIMIT_ENTRIES);
        let keep: std::collections::HashSet<String> =
            entries.into_iter().map(|(k, _)| k).collect();
        attempts.retain(|k, _| keep.contains(k));
    }

    if let Some((count, first_attempt)) = attempts.get(key) {
        if first_attempt.elapsed().as_secs() < LOGIN_WINDOW_SECS && *count >= MAX_LOGIN_ATTEMPTS {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }
    Ok(())
}

fn record_failed_login(key: &str) {
    let mut attempts = get_login_attempts().lock().unwrap();
    let entry = attempts.entry(key.to_string()).or_insert((0, Instant::now()));
    if entry.1.elapsed().as_secs() >= LOGIN_WINDOW_SECS {
        // Reset window
        *entry = (1, Instant::now());
    } else {
        entry.0 += 1;
    }
}

fn clear_login_attempts(key: &str) {
    let mut attempts = get_login_attempts().lock().unwrap();
    attempts.remove(key);
}

// ---------------------------------------------------------------------------
// Input sanitization
// ---------------------------------------------------------------------------

fn sanitize_input(s: &str) -> String {
    static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());
    let trimmed = s.trim();
    RE.replace_all(trimmed, "").to_string()
}

// ---------------------------------------------------------------------------
// Pagination query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation helper
// ---------------------------------------------------------------------------

fn validation_error(field: &str, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": message,
            "field": field,
        })),
    )
}

fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    !trimmed.is_empty() && trimmed.contains('@') && trimmed.len() >= 3
}

fn is_valid_date(date: &str) -> bool {
    // Accepts YYYY-MM-DD format
    if date.len() != 10 {
        return false;
    }
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

// ---------------------------------------------------------------------------
// Auth extractor helper
// ---------------------------------------------------------------------------

pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub role: String,
    pub name: String,
}

pub fn extract_user(headers: &HeaderMap) -> Result<AuthUser, StatusCode> {
    let cookie_header = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = cookie_header
        .split(';')
        .filter_map(|s| {
            let s = s.trim();
            s.strip_prefix("token=")
        })
        .next()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = auth::validate_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(AuthUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
        name: claims.name,
    })
}

pub fn require_auth(headers: &HeaderMap) -> Result<AuthUser, StatusCode> {
    extract_user(headers)
}

pub fn require_landlord(headers: &HeaderMap) -> Result<AuthUser, StatusCode> {
    let user = extract_user(headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(user)
}

// ---------------------------------------------------------------------------
// Row-mapping helpers
// ---------------------------------------------------------------------------

fn payment_from_row(row: &rusqlite::Row) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: row.get(0)?,
        user_id: row.get(1)?,
        amount: row.get(2)?,
        payment_type: row.get(3)?,
        description: row.get(4)?,
        stripe_payment_id: row.get(5)?,
        status: row.get(6)?,
        due_date: row.get(7)?,
        paid_date: row.get(8)?,
        created_at: row.get(9)?,
    })
}

fn utility_from_row(row: &rusqlite::Row) -> rusqlite::Result<UtilityCharge> {
    Ok(UtilityCharge {
        id: row.get(0)?,
        description: row.get(1)?,
        amount: row.get(2)?,
        due_date: row.get(3)?,
        paid: row.get::<_, i32>(4)? != 0,
        payment_id: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn maintenance_from_row(row: &rusqlite::Row) -> rusqlite::Result<MaintenanceRequest> {
    Ok(MaintenanceRequest {
        id: row.get(0)?,
        user_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        status: row.get(4)?,
        photo_path: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// Shared query helpers
// ---------------------------------------------------------------------------

fn query_maintenance_stats(db: &rusqlite::Connection) -> Result<MaintenanceStats, StatusCode> {
    let mut stmt = db
        .prepare("SELECT status, COUNT(*) FROM maintenance_requests GROUP BY status")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut submitted: u32 = 0;
    let mut in_progress: u32 = 0;
    let mut completed: u32 = 0;
    let mut total: u32 = 0;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let (status, count) = row.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        total += count;
        match status.as_str() {
            "submitted" => submitted = count,
            "in_progress" => in_progress = count,
            "completed" => completed = count,
            _ => {}
        }
    }

    Ok(MaintenanceStats {
        total,
        submitted,
        in_progress,
        completed,
    })
}

fn query_monthly_revenue(db: &rusqlite::Connection) -> Result<Vec<RevenueMonth>, StatusCode> {
    let mut stmt = db
        .prepare(
            "SELECT strftime('%Y-%m', paid_date) as month,
                    SUM(amount) as total,
                    SUM(CASE WHEN payment_type = 'rent' THEN amount ELSE 0 END) as rent,
                    SUM(CASE WHEN payment_type = 'utility' THEN amount ELSE 0 END) as utility,
                    COUNT(*) as cnt
             FROM payments
             WHERE status = 'completed' AND paid_date IS NOT NULL
               AND paid_date >= date('now', '-12 months')
             GROUP BY strftime('%Y-%m', paid_date)
             ORDER BY month DESC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let months = stmt
        .query_map([], |row| {
            Ok(RevenueMonth {
                month: row.get(0)?,
                total: row.get(1)?,
                rent: row.get(2)?,
                utility: row.get(3)?,
                count: row.get(4)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(months)
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn api_router() -> Router {
    let auth_routes = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(get_me))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password));

    let tenant_routes = Router::new()
        .route("/dashboard", get(tenant_dashboard))
        .route("/payments", get(tenant_payments))
        .route("/utilities", get(tenant_utilities));

    let admin_tenant_routes = Router::new()
        .route("/", get(admin_list_tenants).post(admin_create_tenant))
        .route("/{id}", delete(admin_delete_tenant));

    let admin_report_routes = Router::new()
        .route("/revenue", get(admin_report_revenue))
        .route("/maintenance", get(admin_report_maintenance))
        .route("/overview", get(admin_report_overview));

    let admin_applicant_routes = Router::new()
        .route("/", get(admin_list_applicants))
        .route("/{id}", patch(admin_update_applicant).delete(admin_delete_applicant));

    let admin_routes = Router::new()
        .route("/dashboard", get(admin_dashboard))
        .route("/payments", get(admin_payments))
        .route("/utilities", post(admin_create_utility))
        .nest("/tenants", admin_tenant_routes)
        .nest("/applicants", admin_applicant_routes)
        .nest("/reports", admin_report_routes);

    let maintenance_routes = Router::new()
        .route("/", get(list_maintenance).post(create_maintenance))
        .route("/{id}", patch(update_maintenance))
        .route("/{id}/messages", get(list_maintenance_messages).post(create_maintenance_message));

    let document_routes = Router::new()
        .route("/", get(list_documents))
        .route("/upload", post(upload_document))
        .route("/file/{id}", get(serve_document_file))
        .route("/{id}", delete(delete_document));

    let payment_routes = Router::new()
        .route("/create-checkout", post(create_checkout))
        .route("/create-utility-checkout", post(create_utility_checkout))
        .route("/webhook", post(stripe_webhook));

    let disclosure_routes = crate::disclosures::disclosures_router();

    let signature_routes = Router::new()
        .route("/", post(create_signature));

    let pdf_routes = Router::new()
        .route("/move-in-checklist", get(pdf_move_in_checklist))
        .route("/lead-paint-disclosure", get(pdf_lead_paint_disclosure))
        .route("/deposit-receipt", get(pdf_deposit_receipt))
        .route("/payment-receipt/{payment_id}", get(pdf_payment_receipt));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/tenant", tenant_routes)
        .nest("/admin", admin_routes)
        .nest("/maintenance", maintenance_routes)
        .nest("/documents", document_routes)
        .nest("/payments", payment_routes)
        .nest("/disclosures", disclosure_routes)
        .nest("/signatures", signature_routes)
        .nest("/pdf", pdf_routes)
        .route("/health", get(health_check))
        .route("/apply", post(public_apply))
}

// ---------------------------------------------------------------------------
// Health check handler
// ---------------------------------------------------------------------------

async fn health_check() -> impl IntoResponse {
    // Initialize start time on first call (idempotent)
    let uptime = server_start_time().elapsed().as_secs();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/app.db".to_string());

    // Check DB is accessible and get file size
    let db_size = match std::fs::metadata(&db_path) {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    };

    // Verify we can actually query the DB
    let db_ok = {
        let db = get_db();
        db.execute_batch("SELECT 1").is_ok()
    };

    let status = if db_ok { "ok" } else { "degraded" };
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "uptime_seconds": uptime,
            "db_size_bytes": db_size,
            "db_path": db_path,
        })),
    )
}

// ---------------------------------------------------------------------------
// Auth handlers
// ---------------------------------------------------------------------------

async fn login(Json(body): Json<LoginRequest>) -> Result<impl IntoResponse, impl IntoResponse> {
    // Validation
    if body.email.trim().is_empty() {
        return Err(validation_error("email", "Email is required"));
    }
    if !is_valid_email(&body.email) {
        return Err(validation_error("email", "Invalid email format"));
    }
    if body.password.is_empty() {
        return Err(validation_error("password", "Password is required"));
    }

    // Rate limiting check
    let rate_limit_key = body.email.trim().to_lowercase();
    check_rate_limit(&rate_limit_key).map_err(|status| {
        (
            status,
            Json(serde_json::json!({"error": "Too many login attempts. Please try again in 15 minutes.", "field": "email"})),
        )
    })?;

    // Scope the DB lock so it is released before generate_token (which also
    // acquires the lock internally to create a session row).
    let (id, email, password_hash, role, name) = {
        let db = get_db();
        let row = db.query_row(
            "SELECT id, email, password_hash, role, name FROM users WHERE email = ?1",
            rusqlite::params![body.email],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        );

        match row {
            Ok(r) => r,
            Err(_) => {
                record_failed_login(&rate_limit_key);
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid credentials", "field": "email"})),
                ))
            }
        }
    };

    if !auth::verify_password(&body.password, &password_hash) {
        record_failed_login(&rate_limit_key);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials", "field": "password"})),
        ));
    }

    // Successful login — clear rate limit counter
    clear_login_attempts(&rate_limit_key);

    let token = auth::generate_token(&id, &email, &role, &name)
        .map_err(|_| validation_error("", "Internal server error"))?;

    let cookie = auth::auth_cookie(&token);

    let user = User {
        id,
        email,
        role,
        name,
    };

    let response = LoginResponse {
        user,
        token: token.clone(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, Json(response)))
}

async fn logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        "token=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"
            .parse()
            .unwrap(),
    );
    (headers, StatusCode::OK)
}

async fn get_me(headers: HeaderMap) -> Result<Json<User>, StatusCode> {
    let user = require_auth(&headers)?;
    Ok(Json(User {
        id: user.id,
        email: user.email,
        role: user.role,
        name: user.name,
    }))
}

// ---------------------------------------------------------------------------
// Forgot / Reset password handlers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

async fn forgot_password(Json(body): Json<ForgotPasswordRequest>) -> Result<impl IntoResponse, impl IntoResponse> {
    // Validation
    if body.email.trim().is_empty() {
        return Err(validation_error("email", "Email is required"));
    }
    if !is_valid_email(&body.email) {
        return Err(validation_error("email", "Invalid email format"));
    }

    // Always return 200 to avoid revealing whether an email exists
    let db = get_db();

    let user_row = db.query_row(
        "SELECT id, email FROM users WHERE email = ?1",
        rusqlite::params![body.email],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    if let Ok((user_id, user_email)) = user_row {
        let token = Uuid::new_v4().to_string();
        let id = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(1))
            .expect("valid timestamp")
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let _ = db.execute(
            "INSERT INTO password_reset_tokens (id, user_id, token, expires_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, user_id, token, expires_at],
        );

        let reset_link = format!(
            "https://properties.mason-wheeler.com/reset-password?token={}",
            token
        );
        let subject = "Password Reset Request";
        let email_body = format!(
            "You requested a password reset.\n\n\
             Click the link below to reset your password:\n\
             {reset_link}\n\n\
             This link expires in 1 hour.\n\n\
             If you did not request this, please ignore this email.\n\n\
             — Mason Wheeler Properties"
        );

        // Fire-and-forget: send email in background
        tokio::spawn(async move {
            if let Err(e) = email::send_email(&user_email, subject, &email_body).await {
                tracing::error!("Failed to send password reset email: {}", e);
            }
        });
    }

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

async fn reset_password(Json(body): Json<ResetPasswordRequest>) -> Result<impl IntoResponse, impl IntoResponse> {
    // Validation
    if body.token.trim().is_empty() {
        return Err(validation_error("token", "Reset token is required"));
    }
    if body.new_password.len() < 8 {
        return Err(validation_error("new_password", "Password must be at least 8 characters"));
    }

    let db = get_db();
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    // Look up the token: must exist, not expired, not used
    let token_row = db.query_row(
        "SELECT id, user_id FROM password_reset_tokens WHERE token = ?1 AND used = 0 AND expires_at > ?2",
        rusqlite::params![body.token, now],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    let (token_id, user_id) = match token_row {
        Ok(r) => r,
        Err(_) => return Err(validation_error("token", "Invalid or expired reset token")),
    };

    // Hash the new password
    let password_hash = auth::hash_password(&body.new_password);

    // Update user's password
    db.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        rusqlite::params![password_hash, user_id],
    )
    .map_err(|_| validation_error("", "Failed to update password"))?;

    // Mark token as used
    let _ = db.execute(
        "UPDATE password_reset_tokens SET used = 1 WHERE id = ?1",
        rusqlite::params![token_id],
    );

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Tenant handlers
// ---------------------------------------------------------------------------

async fn tenant_dashboard(headers: HeaderMap) -> Result<Json<DashboardData>, StatusCode> {
    let user = require_auth(&headers)?;
    let db = get_db();

    let recent_payments = {
        let mut stmt = db
            .prepare(
                "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
                 FROM payments WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 5",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map(rusqlite::params![user.id], |row| payment_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let active_maintenance = {
        let mut stmt = db
            .prepare(
                "SELECT id, user_id, title, description, status, photo_path, created_at, updated_at
                 FROM maintenance_requests WHERE user_id = ?1 AND status != 'completed'
                 ORDER BY created_at DESC",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map(rusqlite::params![user.id], |row| maintenance_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let utility_charges = {
        let mut stmt = db
            .prepare(
                "SELECT id, description, amount, due_date, paid, payment_id, created_at
                 FROM utility_charges WHERE paid = 0 ORDER BY due_date ASC",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map([], |row| utility_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let amount_due: f64 = utility_charges.iter().map(|u| u.amount).sum();

    let next_due_date = utility_charges.first().map(|u| u.due_date.clone());

    Ok(Json(DashboardData {
        amount_due,
        next_due_date,
        recent_payments,
        active_maintenance,
        utility_charges,
    }))
}

async fn tenant_payments(
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Payment>>, StatusCode> {
    let user = require_auth(&headers)?;
    let db = get_db();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut where_clauses = vec!["user_id = ?1".to_string()];
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(user.id.clone())];

    if let Some(ref search) = params.search {
        if !search.is_empty() {
            where_clauses.push(format!("description LIKE ?{}", bind_params.len() + 1));
            bind_params.push(Box::new(format!("%{}%", search)));
        }
    }

    let where_sql = where_clauses.join(" AND ");

    let count_sql = format!("SELECT COUNT(*) FROM payments WHERE {}", where_sql);
    let count_refs: Vec<&dyn rusqlite::types::ToSql> = bind_params.iter().map(|p| p.as_ref()).collect();
    let total: u64 = db
        .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_sql = format!(
        "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
         FROM payments WHERE {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
        where_sql,
        bind_params.len() + 1,
        bind_params.len() + 2,
    );

    let mut query_params = bind_params;
    query_params.push(Box::new(per_page as i64));
    query_params.push(Box::new(offset as i64));
    let query_refs: Vec<&dyn rusqlite::types::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = db.prepare(&query_sql).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let payments = stmt
        .query_map(query_refs.as_slice(), |row| payment_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(PaginatedResponse {
        items: payments,
        total,
        page,
        per_page,
    }))
}

async fn tenant_utilities(headers: HeaderMap) -> Result<Json<Vec<UtilityCharge>>, StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let mut stmt = db
        .prepare(
            "SELECT id, description, amount, due_date, paid, payment_id, created_at
             FROM utility_charges WHERE paid = 0 ORDER BY due_date ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let charges = stmt
        .query_map([], |row| utility_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(charges))
}

// ---------------------------------------------------------------------------
// Admin handlers
// ---------------------------------------------------------------------------

async fn admin_dashboard(headers: HeaderMap) -> Result<Json<DashboardData>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();

    let recent_payments = {
        let mut stmt = db
            .prepare(
                "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
                 FROM payments ORDER BY created_at DESC LIMIT 10",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map([], |row| payment_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let active_maintenance = {
        let mut stmt = db
            .prepare(
                "SELECT id, user_id, title, description, status, photo_path, created_at, updated_at
                 FROM maintenance_requests WHERE status != 'completed'
                 ORDER BY created_at DESC",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map([], |row| maintenance_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let utility_charges = {
        let mut stmt = db
            .prepare(
                "SELECT id, description, amount, due_date, paid, payment_id, created_at
                 FROM utility_charges ORDER BY due_date DESC",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map([], |row| utility_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
    };

    let amount_due: f64 = utility_charges
        .iter()
        .filter(|u| !u.paid)
        .map(|u| u.amount)
        .sum();

    let next_due_date = utility_charges
        .iter()
        .filter(|u| !u.paid)
        .map(|u| u.due_date.clone())
        .min();

    Ok(Json(DashboardData {
        amount_due,
        next_due_date,
        recent_payments,
        active_maintenance,
        utility_charges,
    }))
}

async fn admin_payments(
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Payment>>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut where_clauses: Vec<String> = vec![];
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref search) = params.search {
        if !search.is_empty() {
            where_clauses.push(format!("description LIKE ?{}", bind_params.len() + 1));
            bind_params.push(Box::new(format!("%{}%", search)));
        }
    }

    if let Some(ref status) = params.status {
        if !status.is_empty() && status != "all" {
            where_clauses.push(format!("status = ?{}", bind_params.len() + 1));
            bind_params.push(Box::new(status.clone()));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        "1=1".to_string()
    } else {
        where_clauses.join(" AND ")
    };

    let count_sql = format!("SELECT COUNT(*) FROM payments WHERE {}", where_sql);
    let count_refs: Vec<&dyn rusqlite::types::ToSql> = bind_params.iter().map(|p| p.as_ref()).collect();
    let total: u64 = db
        .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_sql = format!(
        "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
         FROM payments WHERE {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
        where_sql,
        bind_params.len() + 1,
        bind_params.len() + 2,
    );

    let mut query_params = bind_params;
    query_params.push(Box::new(per_page as i64));
    query_params.push(Box::new(offset as i64));
    let query_refs: Vec<&dyn rusqlite::types::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = db.prepare(&query_sql).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let payments = stmt
        .query_map(query_refs.as_slice(), |row| payment_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(PaginatedResponse {
        items: payments,
        total,
        page,
        per_page,
    }))
}

#[derive(Deserialize)]
struct CreateUtilityRequest {
    description: String,
    amount: f64,
    due_date: String,
}

async fn admin_create_utility(
    headers: HeaderMap,
    Json(body): Json<CreateUtilityRequest>,
) -> Result<Json<UtilityCharge>, impl IntoResponse> {
    let _user = require_landlord(&headers).map_err(|s| s.into_response())?;

    let description = sanitize_input(&body.description);

    // Validation
    if description.is_empty() {
        return Err(validation_error("description", "Description is required").into_response());
    }
    if body.amount <= 0.0 {
        return Err(validation_error("amount", "Amount must be greater than 0").into_response());
    }
    if !is_valid_date(&body.due_date) {
        return Err(validation_error("due_date", "Due date must be in YYYY-MM-DD format").into_response());
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = get_db();
    db.execute(
        "INSERT INTO utility_charges (id, description, amount, due_date, paid, created_at) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        rusqlite::params![id, description, body.amount, body.due_date, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    Ok(Json(UtilityCharge {
        id,
        description,
        amount: body.amount,
        due_date: body.due_date,
        paid: false,
        payment_id: None,
        created_at: now,
    }))
}

// ---------------------------------------------------------------------------
// Admin tenant management handlers
// ---------------------------------------------------------------------------

async fn admin_list_tenants(headers: HeaderMap) -> Result<Json<Vec<mason_wheeler_shared::TenantUser>>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();

    let mut stmt = db
        .prepare("SELECT id, email, name, created_at FROM users WHERE role = 'tenant' ORDER BY created_at DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tenants = stmt
        .query_map([], |row| {
            Ok(mason_wheeler_shared::TenantUser {
                id: row.get(0)?,
                email: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(tenants))
}

async fn admin_create_tenant(
    headers: HeaderMap,
    Json(body): Json<mason_wheeler_shared::CreateTenantRequest>,
) -> Result<Json<mason_wheeler_shared::TenantUser>, impl IntoResponse> {
    let _user = require_landlord(&headers).map_err(|s| s.into_response())?;

    let tenant_name = sanitize_input(&body.name);

    // Validation
    if tenant_name.is_empty() {
        return Err(validation_error("name", "Name is required").into_response());
    }
    if body.email.trim().is_empty() {
        return Err(validation_error("email", "Email is required").into_response());
    }
    if !is_valid_email(&body.email) {
        return Err(validation_error("email", "Invalid email format").into_response());
    }
    if body.password.len() < 8 {
        return Err(validation_error("password", "Password must be at least 8 characters").into_response());
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let password_hash = auth::hash_password(&body.password);

    let db = get_db();
    db.execute(
        "INSERT INTO users (id, email, password_hash, role, name, created_at) VALUES (?1, ?2, ?3, 'tenant', ?4, ?5)",
        rusqlite::params![id, body.email, password_hash, tenant_name, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    // Send welcome email (non-blocking)
    {
        let tenant_email = body.email.clone();
        let tn = tenant_name.clone();
        let temp_password = body.password.clone();
        tokio::spawn(async move {
            if let Err(e) =
                email::send_welcome_email(&tenant_email, &tn, &temp_password).await
            {
                tracing::error!("Failed to send welcome email: {e}");
            }
        });
    }

    Ok(Json(mason_wheeler_shared::TenantUser {
        id,
        email: body.email,
        name: tenant_name,
        created_at: now,
    }))
}

async fn admin_delete_tenant(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();
    let rows = db
        .execute(
            "DELETE FROM users WHERE id = ?1 AND role = 'tenant'",
            rusqlite::params![id],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({ "deleted": id })))
}

// ---------------------------------------------------------------------------
// Admin report handlers
// ---------------------------------------------------------------------------

async fn admin_report_revenue(headers: HeaderMap) -> Result<Json<Vec<RevenueMonth>>, StatusCode> {
    let _user = require_landlord(&headers)?;
    let db = get_db();
    let months = query_monthly_revenue(&db)?;
    Ok(Json(months))
}

async fn admin_report_maintenance(headers: HeaderMap) -> Result<Json<MaintenanceStats>, StatusCode> {
    let _user = require_landlord(&headers)?;
    let db = get_db();
    let stats = query_maintenance_stats(&db)?;
    Ok(Json(stats))
}

async fn admin_report_overview(headers: HeaderMap) -> Result<Json<OverviewReport>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();

    let total_collected: f64 = db
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM payments WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_outstanding: f64 = db
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM utility_charges WHERE paid = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_payments: u32 = db
        .query_row("SELECT COUNT(*) FROM payments", [], |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_tenants: u32 = db
        .query_row(
            "SELECT COUNT(*) FROM users WHERE role = 'tenant'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let maintenance_stats = query_maintenance_stats(&db)?;
    let monthly_revenue = query_monthly_revenue(&db)?;

    Ok(Json(OverviewReport {
        total_collected,
        total_outstanding,
        total_payments,
        active_tenants,
        maintenance_stats,
        monthly_revenue,
    }))
}

// ---------------------------------------------------------------------------
// Shared handlers: documents & maintenance
// ---------------------------------------------------------------------------

async fn list_documents(headers: HeaderMap) -> Result<Json<Vec<Document>>, StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let mut stmt = db
        .prepare("SELECT id, name, doc_type, file_path, uploaded_at FROM documents ORDER BY uploaded_at DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let docs = stmt
        .query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                name: row.get(1)?,
                doc_type: row.get(2)?,
                file_path: row.get(3)?,
                uploaded_at: row.get(4)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(docs))
}

async fn upload_document(
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Document>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut original_name: Option<String> = None;
    let mut doc_type: Option<String> = None;
    let mut display_name: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "file" => {
                original_name = field.file_name().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .to_vec(),
                );
            }
            "doc_type" => {
                doc_type =
                    Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "name" => {
                display_name =
                    Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            _ => {}
        }
    }

    let file_data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let doc_type = doc_type.ok_or(StatusCode::BAD_REQUEST)?;
    let display_name = display_name
        .or(original_name.clone())
        .unwrap_or_else(|| "untitled".to_string());

    // Determine file extension from original name
    let extension = original_name
        .as_deref()
        .and_then(|n| n.rsplit('.').next())
        .unwrap_or("bin");

    let file_uuid = Uuid::new_v4().to_string();
    let stored_filename = format!("{}.{}", file_uuid, extension);
    let disk_path = format!("data/documents/{}", stored_filename);

    std::fs::write(&disk_path, &file_data).map_err(|e| {
        tracing::error!("Failed to write uploaded file: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let file_path = format!("/api/documents/file/{}", id);

    let db = get_db();
    db.execute(
        "INSERT INTO documents (id, name, doc_type, file_path, uploaded_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, display_name, doc_type, disk_path, now],
    )
    .map_err(|e| {
        // Clean up the file if DB insert fails
        let _ = std::fs::remove_file(&disk_path);
        tracing::error!("Failed to insert document record: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(Document {
        id,
        name: display_name,
        doc_type,
        file_path,
        uploaded_at: now,
    }))
}

async fn serve_document_file(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let (disk_path, name): (String, String) = db
        .query_row(
            "SELECT file_path, name FROM documents WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let data = std::fs::read(&disk_path).map_err(|e| {
        tracing::error!("Failed to read document file {}: {}", disk_path, e);
        StatusCode::NOT_FOUND
    })?;

    // Guess content type from extension
    let content_type = match disk_path.rsplit('.').next().unwrap_or("") {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "doc" => "application/msword",
        "docx" => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        _ => "application/octet-stream",
    };

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    response_headers.insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", name).parse().unwrap(),
    );

    Ok((response_headers, data))
}

async fn delete_document(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();

    let disk_path: String = db
        .query_row(
            "SELECT file_path FROM documents WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    db.execute(
        "DELETE FROM documents WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Best-effort file deletion
    let _ = std::fs::remove_file(&disk_path);

    Ok(StatusCode::NO_CONTENT)
}


async fn list_maintenance(
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<MaintenanceRequest>>, StatusCode> {
    let user = require_auth(&headers)?;
    let db = get_db();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut where_clauses: Vec<String> = vec![];
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if user.role != "landlord" {
        where_clauses.push(format!("user_id = ?{}", bind_params.len() + 1));
        bind_params.push(Box::new(user.id.clone()) as Box<dyn rusqlite::types::ToSql>);
    }

    if let Some(ref search) = params.search {
        if !search.is_empty() {
            where_clauses.push(format!("title LIKE ?{}", bind_params.len() + 1));
            bind_params.push(Box::new(format!("%{}%", search)));
        }
    }

    if let Some(ref status) = params.status {
        if !status.is_empty() && status != "all" {
            where_clauses.push(format!("status = ?{}", bind_params.len() + 1));
            bind_params.push(Box::new(status.clone()));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        "1=1".to_string()
    } else {
        where_clauses.join(" AND ")
    };

    let count_sql = format!("SELECT COUNT(*) FROM maintenance_requests WHERE {}", where_sql);
    let count_refs: Vec<&dyn rusqlite::types::ToSql> = bind_params.iter().map(|p| p.as_ref()).collect();
    let total: u64 = db
        .query_row(&count_sql, count_refs.as_slice(), |row| row.get(0))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_sql = format!(
        "SELECT id, user_id, title, description, status, photo_path, created_at, updated_at
         FROM maintenance_requests WHERE {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
        where_sql,
        bind_params.len() + 1,
        bind_params.len() + 2,
    );

    let mut query_params = bind_params;
    query_params.push(Box::new(per_page as i64));
    query_params.push(Box::new(offset as i64));
    let query_refs: Vec<&dyn rusqlite::types::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = db.prepare(&query_sql).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let requests = stmt
        .query_map(query_refs.as_slice(), |row| maintenance_from_row(row))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(PaginatedResponse {
        items: requests,
        total,
        page,
        per_page,
    }))
}

#[derive(Deserialize)]
struct CreateMaintenanceRequest {
    title: String,
    description: String,
}

async fn create_maintenance(
    headers: HeaderMap,
    Json(body): Json<CreateMaintenanceRequest>,
) -> Result<Json<MaintenanceRequest>, impl IntoResponse> {
    let user = require_auth(&headers).map_err(|s| s.into_response())?;

    let title = sanitize_input(&body.title);
    let description = sanitize_input(&body.description);

    // Validation
    if title.is_empty() {
        return Err(validation_error("title", "Title is required").into_response());
    }
    if description.is_empty() {
        return Err(validation_error("description", "Description is required").into_response());
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = get_db();
    db.execute(
        "INSERT INTO maintenance_requests (id, user_id, title, description, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'submitted', ?5, ?5)",
        rusqlite::params![id, user.id, title, description, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    Ok(Json(MaintenanceRequest {
        id,
        user_id: user.id,
        title,
        description,
        status: "submitted".to_string(),
        photo_path: None,
        created_at: now.clone(),
        updated_at: now,
    }))
}

#[derive(Deserialize)]
struct UpdateMaintenanceRequest {
    status: String,
}

async fn update_maintenance(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateMaintenanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user = require_landlord(&headers)?;
    let now = chrono::Utc::now().to_rfc3339();

    {
        let db = get_db();
        let rows = db
            .execute(
                "UPDATE maintenance_requests SET status = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![body.status, now, id],
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if rows == 0 {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    // Send maintenance update email (non-blocking)
    {
        let db2 = get_db();
        if let Ok((tenant_email, title)) = db2.query_row(
            "SELECT u.email, m.title FROM maintenance_requests m
             JOIN users u ON u.id = m.user_id
             WHERE m.id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ) {
            let status = body.status.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    email::send_maintenance_update(&tenant_email, &title, &status).await
                {
                    tracing::error!("Failed to send maintenance update email: {e}");
                }
            });
        }
    }

    Ok(Json(serde_json::json!({ "id": id, "status": body.status })))
}

// ---------------------------------------------------------------------------
// Maintenance messages
// ---------------------------------------------------------------------------

async fn list_maintenance_messages(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Vec<MaintenanceMessage>>, StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let mut stmt = db
        .prepare(
            "SELECT mm.id, mm.request_id, mm.user_id, u.name, mm.message, mm.created_at
             FROM maintenance_messages mm
             JOIN users u ON u.id = mm.user_id
             WHERE mm.request_id = ?1
             ORDER BY mm.created_at ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let messages = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok(MaintenanceMessage {
                id: row.get(0)?,
                request_id: row.get(1)?,
                user_id: row.get(2)?,
                user_name: row.get(3)?,
                message: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(messages))
}

#[derive(Deserialize)]
struct CreateMessageRequest {
    message: String,
}

async fn create_maintenance_message(
    headers: HeaderMap,
    Path(request_id): Path<String>,
    Json(body): Json<CreateMessageRequest>,
) -> Result<Json<MaintenanceMessage>, StatusCode> {
    let user = require_auth(&headers)?;
    let message = sanitize_input(&body.message);
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = get_db();
    db.execute(
        "INSERT INTO maintenance_messages (id, request_id, user_id, message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, request_id, user.id, message, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MaintenanceMessage {
        id,
        request_id,
        user_id: user.id,
        user_name: Some(user.name),
        message,
        created_at: now,
    }))
}

// ---------------------------------------------------------------------------
// Payment routes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CheckoutRequest {
    /// Amount in dollars; defaults to 3250.00 (monthly rent) if omitted.
    amount: Option<f64>,
    /// Optional description for the line item.
    description: Option<String>,
}

#[derive(Serialize)]
struct CheckoutResponse {
    url: String,
}

async fn create_checkout(
    headers: HeaderMap,
    body: Option<Json<CheckoutRequest>>,
) -> Result<Json<CheckoutResponse>, StatusCode> {
    let user = require_auth(&headers)?;

    let stripe_secret =
        std::env::var("STRIPE_SECRET_KEY").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let amount_dollars = body
        .as_ref()
        .and_then(|b| b.amount)
        .unwrap_or(3250.00);
    let description = body
        .as_ref()
        .and_then(|b| b.description.clone())
        .unwrap_or_else(|| "Rent Payment".to_string());

    // Stripe expects amounts in cents
    let amount_cents = (amount_dollars * 100.0).round() as i64;

    let client = stripe::Client::new(&stripe_secret);

    let success_url = "https://properties.mason-wheeler.com/tenant/payments?success=true";
    let cancel_url = "https://properties.mason-wheeler.com/tenant/payments?cancelled=true";

    let mut params = stripe::CreateCheckoutSession::new();
    params.success_url = Some(success_url);
    params.cancel_url = Some(cancel_url);
    params.mode = Some(stripe::CheckoutSessionMode::Payment);
    params.payment_method_types = Some(vec![
        stripe::CreateCheckoutSessionPaymentMethodTypes::UsBankAccount,
        stripe::CreateCheckoutSessionPaymentMethodTypes::Card,
    ]);
    params.line_items = Some(vec![stripe::CreateCheckoutSessionLineItems {
        price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
            currency: stripe::Currency::USD,
            product_data: Some(
                stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: description.clone(),
                    ..Default::default()
                },
            ),
            unit_amount: Some(amount_cents),
            ..Default::default()
        }),
        quantity: Some(1),
        ..Default::default()
    }]);
    params.metadata = Some(
        [
            ("user_id".to_string(), user.id.clone()),
            ("description".to_string(), description),
        ]
        .into_iter()
        .collect(),
    );

    let session = stripe::CheckoutSession::create(&client, params)
        .await
        .map_err(|e| {
            tracing::error!("Stripe checkout session creation failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let url = session.url.ok_or_else(|| {
        tracing::error!("Stripe returned no checkout URL");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CheckoutResponse { url }))
}

// ---------------------------------------------------------------------------
// Utility charge checkout
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct UtilityCheckoutRequest {
    utility_charge_id: String,
}

async fn create_utility_checkout(
    headers: HeaderMap,
    Json(body): Json<UtilityCheckoutRequest>,
) -> Result<Json<CheckoutResponse>, StatusCode> {
    let user = require_auth(&headers)?;

    let stripe_secret =
        std::env::var("STRIPE_SECRET_KEY").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Look up the utility charge from DB (scope the lock so it's dropped before await)
    let (charge_id, description, amount_cents) = {
        let db = get_db();
        let charge = db
            .query_row(
                "SELECT id, description, amount, paid FROM utility_charges WHERE id = ?1",
                rusqlite::params![body.utility_charge_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, bool>(3)?,
                    ))
                },
            )
            .map_err(|_| StatusCode::NOT_FOUND)?;

        let (charge_id, description, amount, paid) = charge;

        if paid {
            return Err(StatusCode::BAD_REQUEST);
        }

        let amount_cents = (amount * 100.0).round() as i64;
        (charge_id, description, amount_cents)
    };

    let client = stripe::Client::new(&stripe_secret);

    let success_url = "https://properties.mason-wheeler.com/tenant/payments?success=true";
    let cancel_url = "https://properties.mason-wheeler.com/tenant/payments?cancelled=true";

    let mut params = stripe::CreateCheckoutSession::new();
    params.success_url = Some(success_url);
    params.cancel_url = Some(cancel_url);
    params.mode = Some(stripe::CheckoutSessionMode::Payment);
    params.payment_method_types = Some(vec![
        stripe::CreateCheckoutSessionPaymentMethodTypes::UsBankAccount,
        stripe::CreateCheckoutSessionPaymentMethodTypes::Card,
    ]);
    params.line_items = Some(vec![stripe::CreateCheckoutSessionLineItems {
        price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
            currency: stripe::Currency::USD,
            product_data: Some(
                stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: description.clone(),
                    ..Default::default()
                },
            ),
            unit_amount: Some(amount_cents),
            ..Default::default()
        }),
        quantity: Some(1),
        ..Default::default()
    }]);
    params.metadata = Some(
        [
            ("user_id".to_string(), user.id.clone()),
            ("description".to_string(), description),
            ("utility_charge_id".to_string(), charge_id),
        ]
        .into_iter()
        .collect(),
    );

    let session = stripe::CheckoutSession::create(&client, params)
        .await
        .map_err(|e| {
            tracing::error!("Stripe utility checkout session creation failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let url = session.url.ok_or_else(|| {
        tracing::error!("Stripe returned no checkout URL");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CheckoutResponse { url }))
}

// ---------------------------------------------------------------------------
// Stripe webhook
// ---------------------------------------------------------------------------

async fn stripe_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let webhook_secret =
        std::env::var("STRIPE_WEBHOOK_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sig = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let payload = std::str::from_utf8(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    let event = stripe::Webhook::construct_event(payload, sig, &webhook_secret).map_err(|e| {
        tracing::error!("Webhook signature verification failed: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    if event.type_ == stripe::EventType::CheckoutSessionCompleted {
        if let stripe::EventObject::CheckoutSession(session) = event.data.object {
            let amount = session.amount_total.unwrap_or(0) as f64 / 100.0;
            let stripe_payment_id = session.payment_intent.map(|pi| match pi {
                stripe::Expandable::Id(id) => id.to_string(),
                stripe::Expandable::Object(obj) => obj.id.to_string(),
            });
            let user_id = session
                .metadata
                .as_ref()
                .and_then(|m| m.get("user_id").cloned());
            let description = session
                .metadata
                .as_ref()
                .and_then(|m| m.get("description").cloned())
                .unwrap_or_else(|| "Rent Payment".to_string());

            let payment_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            // If this payment is for a utility charge, extract the id before scoping the db lock
            let utility_charge_id = session
                .metadata
                .as_ref()
                .and_then(|m| m.get("utility_charge_id").cloned());

            {
                let db = get_db();

                // Idempotency: skip if a payment with this stripe_payment_id already exists
                if let Some(ref spid) = stripe_payment_id {
                    let exists: bool = db
                        .query_row(
                            "SELECT COUNT(*) FROM payments WHERE stripe_payment_id = ?1",
                            rusqlite::params![spid],
                            |row| row.get::<_, u32>(0),
                        )
                        .map(|c| c > 0)
                        .unwrap_or(false);
                    if exists {
                        tracing::info!(
                            stripe_payment_id = %spid,
                            "Duplicate webhook event — payment already recorded, skipping"
                        );
                        return Ok(Json(serde_json::json!({ "received": true })));
                    }
                }

                let result = db.execute(
                    "INSERT INTO payments (id, user_id, amount, payment_type, description, stripe_payment_id, status, paid_date, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'completed', ?7, ?7)",
                    rusqlite::params![
                        payment_id,
                        user_id,
                        amount,
                        "stripe",
                        description,
                        stripe_payment_id,
                        now,
                    ],
                );

                if let Err(e) = result {
                    tracing::error!("Failed to record payment from webhook: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }

                if let Some(ref charge_id) = utility_charge_id {
                    let result = db.execute(
                        "UPDATE utility_charges SET paid = 1, payment_id = ?1 WHERE id = ?2",
                        rusqlite::params![payment_id, charge_id],
                    );
                    if let Err(e) = result {
                        tracing::error!("Failed to mark utility charge {} as paid: {}", charge_id, e);
                    } else {
                        tracing::info!(
                            utility_charge_id = %charge_id,
                            payment_id = %payment_id,
                            "Utility charge marked as paid"
                        );
                    }
                }
            } // db lock dropped here

            tracing::info!(
                payment_id = %payment_id,
                amount = %amount,
                "Payment recorded from checkout.session.completed"
            );

            // Send payment confirmation email (non-blocking)
            if let Some(ref uid) = user_id {
                let db2 = get_db();
                if let Ok(tenant_email) = db2.query_row(
                    "SELECT email FROM users WHERE id = ?1",
                    rusqlite::params![uid],
                    |row| row.get::<_, String>(0),
                ) {
                    let paid_date = now.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            email::send_payment_confirmation(&tenant_email, amount, &paid_date)
                                .await
                        {
                            tracing::error!("Failed to send payment confirmation email: {e}");
                        }
                    });
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "received": true })))
}

// ---------------------------------------------------------------------------
// Signature handlers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CreateSignatureRequest {
    document_id: String,
    document_type: String,
    signature_data: String,
    signer_role: String,
}

async fn create_signature(
    headers: HeaderMap,
    Json(body): Json<CreateSignatureRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = require_auth(&headers)?;

    // Validate signer_role
    if body.signer_role != "tenant" && body.signer_role != "landlord" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Strip the data URL prefix if present, then decode base64 PNG
    let base64_data = body
        .signature_data
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&body.signature_data);

    use base64::Engine;
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let sig_id = Uuid::new_v4().to_string();
    let file_name = format!("{}.png", sig_id);
    let file_path = format!("data/signatures/{}", file_name);

    std::fs::write(&file_path, &png_bytes).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let db = get_db();

    // Insert into signatures table
    db.execute(
        "INSERT INTO signatures (id, document_type, document_id, signer_role, signer_name, signature_path)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            sig_id,
            body.document_type,
            body.document_id,
            body.signer_role,
            user.name,
            file_path,
        ],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update the relevant document's signed field based on document_type
    let signed_column = if body.signer_role == "tenant" {
        "tenant_signed"
    } else {
        "landlord_signed"
    };

    match body.document_type.as_str() {
        "move_in_checklist" => {
            let query = format!(
                "UPDATE move_in_checklists SET {} = 1 WHERE id = ?1",
                signed_column
            );
            db.execute(&query, rusqlite::params![body.document_id])
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        "lead_paint_disclosure" => {
            let col = if body.signer_role == "tenant" {
                "tenant_acknowledged"
            } else {
                "landlord_signed"
            };
            let query = format!(
                "UPDATE lead_paint_disclosures SET {} = 1 WHERE id = ?1",
                col
            );
            db.execute(&query, rusqlite::params![body.document_id])
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        _ => {
            // Unknown document type — signature is still saved, just no table update
        }
    }

    Ok(Json(serde_json::json!({
        "id": sig_id,
        "signature_path": file_path,
    })))
}

// ---------------------------------------------------------------------------
// PDF download handlers
// ---------------------------------------------------------------------------

fn pdf_response(filename: &str, data: Vec<u8>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/pdf".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
    );
    (StatusCode::OK, headers, data)
}

async fn pdf_move_in_checklist(
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    // Fetch most recent checklist
    let checklist_row = db
        .query_row(
            "SELECT id, property_address, tenant_name, landlord_name, move_in_date, tenant_signed, landlord_signed, created_at
             FROM move_in_checklists ORDER BY created_at DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, bool>(5)?,
                    row.get::<_, bool>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let (id, address, tenant, landlord, date, t_signed, l_signed, created) = checklist_row;

    let mut stmt = db
        .prepare(
            "SELECT id, room, item, condition, notes, photo_path, created_at
             FROM checklist_items WHERE checklist_id = ?1 ORDER BY rowid ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<ChecklistItem> = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok(ChecklistItem {
                id: row.get(0)?,
                room: row.get(1)?,
                item: row.get(2)?,
                condition: row.get(3)?,
                notes: row.get(4)?,
                photo_path: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect();

    let mut room_map: std::collections::BTreeMap<String, Vec<ChecklistItem>> =
        std::collections::BTreeMap::new();
    for item in items {
        room_map.entry(item.room.clone()).or_default().push(item);
    }
    let rooms: Vec<ChecklistRoom> = room_map
        .into_iter()
        .map(|(name, items)| ChecklistRoom { name, items })
        .collect();

    let checklist = MoveInChecklist {
        id,
        property_address: address,
        tenant_name: tenant,
        landlord_name: landlord,
        move_in_date: date,
        tenant_signed: t_signed,
        landlord_signed: l_signed,
        rooms,
        created_at: created,
    };

    let json_data =
        serde_json::to_string(&checklist).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = pdf::generate_move_in_checklist_pdf(&json_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(pdf_response("move-in-checklist.pdf", data))
}

async fn pdf_lead_paint_disclosure(
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let row = db
        .query_row(
            "SELECT property_address, year_built, landlord_name, tenant_name
             FROM lead_paint_disclosures ORDER BY created_at DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let (property_address, year_built, landlord_name, tenant_name) = row;

    let data = pdf::generate_lead_paint_disclosure_pdf(
        &property_address,
        year_built,
        &landlord_name,
        &tenant_name,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(pdf_response("lead-paint-disclosure.pdf", data))
}

async fn pdf_deposit_receipt(
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let row = db
        .query_row(
            "SELECT tenant_name, deposit_amount, deposit_type, depository_name, date_received
             FROM deposit_receipts ORDER BY created_at DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let (tenant_name, amount, deposit_type, depository, date) = row;

    let data =
        pdf::generate_deposit_receipt_pdf(&tenant_name, amount, &deposit_type, &depository, &date)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(pdf_response("deposit-receipt.pdf", data))
}

async fn pdf_payment_receipt(
    headers: HeaderMap,
    Path(payment_id): Path<String>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), StatusCode> {
    let _user = require_auth(&headers)?;
    let db = get_db();

    let row = db
        .query_row(
            "SELECT p.amount, p.payment_type, p.description, p.paid_date, p.created_at, u.name
             FROM payments p
             JOIN users u ON p.user_id = u.id
             WHERE p.id = ?1",
            rusqlite::params![payment_id],
            |row| {
                Ok((
                    row.get::<_, f64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let (amount, payment_type, description, paid_date, created_at, tenant_name) = row;
    let date = paid_date.unwrap_or(created_at);
    let desc = description.unwrap_or_default();

    let data =
        pdf::generate_payment_receipt_pdf(&tenant_name, amount, &payment_type, &date, &desc)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(pdf_response(
        &format!("payment-receipt-{}.pdf", payment_id),
        data,
    ))
}

// ---------------------------------------------------------------------------
// Public apply (no auth)
// ---------------------------------------------------------------------------

async fn public_apply(
    Json(body): Json<mason_wheeler_shared::CreateApplicantRequest>,
) -> impl IntoResponse {
    let name = sanitize_input(&body.name);
    let email_raw = body.email.trim().to_string();

    if name.is_empty() {
        return validation_error("name", "Name is required").into_response();
    }
    if email_raw.is_empty() || !email_raw.contains('@') {
        return validation_error("email", "A valid email is required").into_response();
    }

    let id = Uuid::new_v4().to_string();
    let phone = body.phone.as_deref().map(|s| sanitize_input(s));
    let desired_move_in = body.desired_move_in.as_deref().map(|s| sanitize_input(s));
    let message = body.message.as_deref().map(|s| sanitize_input(s));

    // DB operation in its own block so MutexGuard is dropped before any .await
    {
        let db = get_db();
        if db.execute(
            "INSERT INTO applicants (id, name, email, phone, desired_move_in, message) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, name, email_raw, phone, desired_move_in, message],
        ).is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to save application"}))).into_response();
        }
    }

    // Notify landlord via email
    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_default();
    if !admin_email.is_empty() {
        let email_body = format!(
            "New rental application received:\n\n\
             Name: {name}\n\
             Email: {email_raw}\n\
             Phone: {phone}\n\
             Desired move-in: {move_in}\n\
             Message: {msg}\n",
            phone = phone.as_deref().unwrap_or("N/A"),
            move_in = desired_move_in.as_deref().unwrap_or("N/A"),
            msg = message.as_deref().unwrap_or("N/A"),
        );
        if let Err(e) = email::send_email(&admin_email, "New Rental Application", &email_body).await {
            tracing::warn!("Failed to send applicant notification email: {e}");
        }
    }

    (StatusCode::OK, Json(serde_json::json!({"message": "Application received"}))).into_response()
}

// ---------------------------------------------------------------------------
// Admin applicant management
// ---------------------------------------------------------------------------

async fn admin_list_applicants(headers: HeaderMap) -> Result<Json<Vec<mason_wheeler_shared::Applicant>>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();
    let mut stmt = db
        .prepare("SELECT id, name, email, phone, desired_move_in, message, status, notes, created_at FROM applicants ORDER BY created_at DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let applicants = stmt
        .query_map([], |row| {
            Ok(mason_wheeler_shared::Applicant {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                phone: row.get(3)?,
                desired_move_in: row.get(4)?,
                message: row.get(5)?,
                status: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(applicants))
}

#[derive(Debug, Deserialize)]
struct UpdateApplicantRequest {
    status: Option<String>,
    notes: Option<String>,
}

async fn admin_update_applicant(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateApplicantRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();

    // Verify applicant exists
    let exists: bool = db
        .prepare("SELECT 1 FROM applicants WHERE id = ?1")
        .and_then(|mut s| s.exists(rusqlite::params![id]))
        .unwrap_or(false);
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }

    if let Some(ref status) = body.status {
        db.execute(
            "UPDATE applicants SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, id],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(ref notes) = body.notes {
        db.execute(
            "UPDATE applicants SET notes = ?1 WHERE id = ?2",
            rusqlite::params![notes, id],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({"message": "Applicant updated"})))
}

async fn admin_delete_applicant(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user = require_landlord(&headers)?;

    let db = get_db();
    let rows = db
        .execute("DELETE FROM applicants WHERE id = ?1", rusqlite::params![id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(serde_json::json!({"message": "Applicant deleted"})))
}
