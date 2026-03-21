use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard, OnceLock};

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn get_db() -> MutexGuard<'static, Connection> {
    DB.get()
        .expect("Database not initialized. Call init_db() first.")
        .lock()
        .expect("Failed to acquire database lock")
}

pub fn init_db() {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/app.db".to_string());

    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    let conn = Connection::open(&db_path).expect("Failed to open database");

    conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
    conn.execute_batch("PRAGMA foreign_keys=ON;").ok();

    create_tables(&conn);
    seed_default_landlord(&conn);

    DB.set(Mutex::new(conn))
        .expect("Database already initialized");
}

fn create_tables(conn: &Connection) {
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
        ",
    )
    .expect("Failed to create tables");
}

fn seed_default_landlord(conn: &Connection) {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE role = 'landlord'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if count == 0 {
        let email = std::env::var("ADMIN_EMAIL")
            .expect("ADMIN_EMAIL env var required for initial setup");
        let password = std::env::var("ADMIN_PASSWORD")
            .expect("ADMIN_PASSWORD env var required for initial setup");
        let name = std::env::var("ADMIN_NAME")
            .unwrap_or_else(|_| "Landlord".to_string());

        let password_hash = bcrypt::hash(&password, 10).expect("Failed to hash password");

        conn.execute(
            "INSERT INTO users (id, email, password_hash, role, name) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["landlord-1", email, password_hash, "landlord", name],
        )
        .expect("Failed to seed default landlord user");
    }

    // Seed default landlord contact info
    let contact_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM landlord_contact WHERE id = 'main'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if contact_count == 0 {
        let name = std::env::var("ADMIN_NAME")
            .unwrap_or_else(|_| "Landlord".to_string());
        let email = std::env::var("ADMIN_EMAIL")
            .unwrap_or_else(|_| "admin@example.com".to_string());
        let mailing_address = std::env::var("LANDLORD_MAILING_ADDRESS")
            .unwrap_or_else(|_| "".to_string());

        conn.execute(
            "INSERT INTO landlord_contact (id, name, mailing_address, phone, email) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["main", name, mailing_address, "", email],
        )
        .expect("Failed to seed landlord contact info");
    }
}
