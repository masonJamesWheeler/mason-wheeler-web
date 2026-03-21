use axum::{
    body::Bytes,
    extract::{Multipart, Path},
    http::{header, header::SET_COOKIE, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use mason_wheeler_shared::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth;
use crate::db::get_db;
use crate::email;

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

    // Look up user name from DB
    let db = get_db();
    let name: String = db
        .query_row(
            "SELECT name FROM users WHERE id = ?1",
            rusqlite::params![claims.sub],
            |row| row.get(0),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(AuthUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
        name,
    })
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn api_router() -> Router {
    let auth_routes = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password));

    let tenant_routes = Router::new()
        .route("/dashboard", get(tenant_dashboard))
        .route("/payments", get(tenant_payments))
        .route("/utilities", get(tenant_utilities));

    let admin_tenant_routes = Router::new()
        .route("/", get(admin_list_tenants).post(admin_create_tenant))
        .route("/{id}", delete(admin_delete_tenant));

    let admin_routes = Router::new()
        .route("/dashboard", get(admin_dashboard))
        .route("/payments", get(admin_payments))
        .route("/utilities", post(admin_create_utility))
        .nest("/tenants", admin_tenant_routes);

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
        .route("/webhook", post(stripe_webhook));

    let disclosure_routes = crate::disclosures::disclosures_router();

    let signature_routes = Router::new()
        .route("/", post(create_signature));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/tenant", tenant_routes)
        .nest("/admin", admin_routes)
        .nest("/maintenance", maintenance_routes)
        .nest("/documents", document_routes)
        .nest("/payments", payment_routes)
        .nest("/disclosures", disclosure_routes)
        .nest("/signatures", signature_routes)
}

// ---------------------------------------------------------------------------
// Auth handlers
// ---------------------------------------------------------------------------

async fn login(Json(body): Json<LoginRequest>) -> impl IntoResponse {
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

    let (id, email, password_hash, role, name) = match row {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    if !auth::verify_password(&body.password, &password_hash) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = auth::generate_token(&id, &email, &role).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

// ---------------------------------------------------------------------------
// Forgot / Reset password handlers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

async fn forgot_password(Json(body): Json<ForgotPasswordRequest>) -> impl IntoResponse {
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

    StatusCode::OK
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

async fn reset_password(Json(body): Json<ResetPasswordRequest>) -> impl IntoResponse {
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
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Hash the new password
    let password_hash = auth::hash_password(&body.new_password);

    // Update user's password
    db.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        rusqlite::params![password_hash, user_id],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    let user = extract_user(&headers)?;
    let db = get_db();

    let recent_payments = {
        let mut stmt = db
            .prepare(
                "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
                 FROM payments WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 5",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map(rusqlite::params![user.id], |row| {
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
        })
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

        stmt.query_map(rusqlite::params![user.id], |row| {
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
        })
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

        stmt.query_map([], |row| {
            Ok(UtilityCharge {
                id: row.get(0)?,
                description: row.get(1)?,
                amount: row.get(2)?,
                due_date: row.get(3)?,
                paid: row.get::<_, i32>(4)? != 0,
                payment_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
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

async fn tenant_payments(headers: HeaderMap) -> Result<Json<Vec<Payment>>, StatusCode> {
    let user = extract_user(&headers)?;
    let db = get_db();

    let mut stmt = db
        .prepare(
            "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
             FROM payments WHERE user_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payments = stmt
        .query_map(rusqlite::params![user.id], |row| {
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
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(payments))
}

async fn tenant_utilities(headers: HeaderMap) -> Result<Json<Vec<UtilityCharge>>, StatusCode> {
    let _user = extract_user(&headers)?;
    let db = get_db();

    let mut stmt = db
        .prepare(
            "SELECT id, description, amount, due_date, paid, payment_id, created_at
             FROM utility_charges WHERE paid = 0 ORDER BY due_date ASC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let charges = stmt
        .query_map([], |row| {
            Ok(UtilityCharge {
                id: row.get(0)?,
                description: row.get(1)?,
                amount: row.get(2)?,
                due_date: row.get(3)?,
                paid: row.get::<_, i32>(4)? != 0,
                payment_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(charges))
}

// ---------------------------------------------------------------------------
// Admin handlers
// ---------------------------------------------------------------------------

async fn admin_dashboard(headers: HeaderMap) -> Result<Json<DashboardData>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = get_db();

    let recent_payments = {
        let mut stmt = db
            .prepare(
                "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
                 FROM payments ORDER BY created_at DESC LIMIT 10",
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map([], |row| {
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
        })
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

        stmt.query_map([], |row| {
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
        })
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

        stmt.query_map([], |row| {
            Ok(UtilityCharge {
                id: row.get(0)?,
                description: row.get(1)?,
                amount: row.get(2)?,
                due_date: row.get(3)?,
                paid: row.get::<_, i32>(4)? != 0,
                payment_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
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

async fn admin_payments(headers: HeaderMap) -> Result<Json<Vec<Payment>>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = get_db();

    let mut stmt = db
        .prepare(
            "SELECT id, user_id, amount, payment_type, description, stripe_payment_id, status, due_date, paid_date, created_at
             FROM payments ORDER BY created_at DESC",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payments = stmt
        .query_map([], |row| {
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
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(payments))
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
) -> Result<Json<UtilityCharge>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = get_db();
    db.execute(
        "INSERT INTO utility_charges (id, description, amount, due_date, paid, created_at) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        rusqlite::params![id, body.description, body.amount, body.due_date, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UtilityCharge {
        id,
        description: body.description,
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
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

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
) -> Result<Json<mason_wheeler_shared::TenantUser>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let password_hash = auth::hash_password(&body.password);

    let db = get_db();
    db.execute(
        "INSERT INTO users (id, email, password_hash, role, name, created_at) VALUES (?1, ?2, ?3, 'tenant', ?4, ?5)",
        rusqlite::params![id, body.email, password_hash, body.name, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Send welcome email (non-blocking)
    {
        let tenant_email = body.email.clone();
        let tenant_name = body.name.clone();
        let temp_password = body.password.clone();
        tokio::spawn(async move {
            if let Err(e) =
                email::send_welcome_email(&tenant_email, &tenant_name, &temp_password).await
            {
                tracing::error!("Failed to send welcome email: {e}");
            }
        });
    }

    Ok(Json(mason_wheeler_shared::TenantUser {
        id,
        email: body.email,
        name: body.name,
        created_at: now,
    }))
}

async fn admin_delete_tenant(
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

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
// Shared handlers: documents & maintenance
// ---------------------------------------------------------------------------

async fn list_documents(headers: HeaderMap) -> Result<Json<Vec<Document>>, StatusCode> {
    let _user = extract_user(&headers)?;
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
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

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
    let _user = extract_user(&headers)?;
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
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

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


async fn list_maintenance(headers: HeaderMap) -> Result<Json<Vec<MaintenanceRequest>>, StatusCode> {
    let user = extract_user(&headers)?;
    let db = get_db();

    let (query, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if user.role == "landlord" {
        (
            "SELECT id, user_id, title, description, status, photo_path, created_at, updated_at
             FROM maintenance_requests ORDER BY created_at DESC",
            vec![],
        )
    } else {
        (
            "SELECT id, user_id, title, description, status, photo_path, created_at, updated_at
             FROM maintenance_requests WHERE user_id = ?1 ORDER BY created_at DESC",
            vec![Box::new(user.id) as Box<dyn rusqlite::types::ToSql>],
        )
    };

    let mut stmt = db.prepare(query).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let requests = stmt
        .query_map(params_ref.as_slice(), |row| {
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
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(Json(requests))
}

#[derive(Deserialize)]
struct CreateMaintenanceRequest {
    title: String,
    description: String,
}

async fn create_maintenance(
    headers: HeaderMap,
    Json(body): Json<CreateMaintenanceRequest>,
) -> Result<Json<MaintenanceRequest>, StatusCode> {
    let user = extract_user(&headers)?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = get_db();
    db.execute(
        "INSERT INTO maintenance_requests (id, user_id, title, description, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'submitted', ?5, ?5)",
        rusqlite::params![id, user.id, body.title, body.description, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MaintenanceRequest {
        id,
        user_id: user.id,
        title: body.title,
        description: body.description,
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
    let _user = extract_user(&headers)?;
    let now = chrono::Utc::now().to_rfc3339();

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
    let _user = extract_user(&headers)?;
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
    let user = extract_user(&headers)?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let db = get_db();
    db.execute(
        "INSERT INTO maintenance_messages (id, request_id, user_id, message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, request_id, user.id, body.message, now],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MaintenanceMessage {
        id,
        request_id,
        user_id: user.id,
        user_name: Some(user.name),
        message: body.message,
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
    let user = extract_user(&headers)?;

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

            let db = get_db();
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
    let user = extract_user(&headers)?;

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
