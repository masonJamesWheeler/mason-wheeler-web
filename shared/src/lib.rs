use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub bedrooms: i32,
    pub bathrooms: i32,
    pub sqft: i32,
    pub rent: f64,
    pub deposit: Option<f64>,
    pub available_date: Option<String>,
    pub pets_allowed: bool,
    pub lease_term_months: i32,
    pub description: Option<String>,
    pub is_occupied: bool,
    pub year_built: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub role: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub user_id: Option<String>,
    pub amount: f64,
    pub payment_type: String,
    pub description: Option<String>,
    pub stripe_payment_id: Option<String>,
    pub status: String,
    pub due_date: Option<String>,
    pub paid_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityCharge {
    pub id: String,
    pub description: String,
    pub amount: f64,
    pub due_date: String,
    pub paid: bool,
    pub payment_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub photo_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceMessage {
    pub id: String,
    pub request_id: String,
    pub user_id: String,
    pub user_name: Option<String>,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub doc_type: String,
    pub file_path: String,
    pub uploaded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub next_due_date: Option<String>,
    pub amount_due: f64,
    pub recent_payments: Vec<Payment>,
    pub active_maintenance: Vec<MaintenanceRequest>,
    pub utility_charges: Vec<UtilityCharge>,
}

// ---------------------------------------------------------------------------
// Move-In Checklist
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub room: String,
    pub item: String,
    pub condition: String, // "good", "fair", "poor", "damaged", "n/a"
    pub notes: Option<String>,
    pub photo_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistRoom {
    pub name: String,
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveInChecklist {
    pub id: String,
    pub property_address: String,
    pub tenant_name: String,
    pub landlord_name: String,
    pub move_in_date: String,
    pub tenant_signed: bool,
    pub landlord_signed: bool,
    pub rooms: Vec<ChecklistRoom>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChecklistRequest {
    pub tenant_name: String,
    pub move_in_date: String,
    pub items: Vec<CreateChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChecklistItem {
    pub room: String,
    pub item: String,
    pub condition: String,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Legal Disclosures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadPaintDisclosure {
    pub property_address: String,
    pub year_built: i32,
    pub known_lead_paint: bool,
    pub known_hazards_description: Option<String>,
    pub records_available: bool,
    pub records_description: Option<String>,
    pub tenant_name: String,
    pub landlord_name: String,
    pub tenant_acknowledged: bool,
    pub landlord_signed: bool,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositReceipt {
    pub tenant_name: String,
    pub property_address: String,
    pub deposit_amount: f64,
    pub deposit_type: String, // "security", "pet", "last_month"
    pub depository_name: String,
    pub depository_address: String,
    pub date_received: String,
    pub landlord_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandlordContactInfo {
    pub name: String,
    pub mailing_address: String,
    pub phone: String,
    pub email: String,
    pub emergency_contact: Option<String>,
    pub emergency_phone: Option<String>,
}
