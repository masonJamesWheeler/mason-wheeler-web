use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Typed enums (replace stringly-typed fields)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Landlord,
    Tenant,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Landlord => write!(f, "landlord"),
            UserRole::Tenant => write!(f, "tenant"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicantStatus {
    New,
    Screening,
    Approved,
    Denied,
    LeaseSigned,
}

impl std::fmt::Display for ApplicantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicantStatus::New => write!(f, "new"),
            ApplicantStatus::Screening => write!(f, "screening"),
            ApplicantStatus::Approved => write!(f, "approved"),
            ApplicantStatus::Denied => write!(f, "denied"),
            ApplicantStatus::LeaseSigned => write!(f, "lease_signed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceStatus {
    Submitted,
    InProgress,
    Completed,
}

impl std::fmt::Display for MaintenanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaintenanceStatus::Submitted => write!(f, "submitted"),
            MaintenanceStatus::InProgress => write!(f, "in_progress"),
            MaintenanceStatus::Completed => write!(f, "completed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    Rent,
    Utility,
    Deposit,
    LateFee,
    Stripe,
    Manual,
}

impl std::fmt::Display for PaymentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentType::Rent => write!(f, "rent"),
            PaymentType::Utility => write!(f, "utility"),
            PaymentType::Deposit => write!(f, "deposit"),
            PaymentType::LateFee => write!(f, "late_fee"),
            PaymentType::Stripe => write!(f, "stripe"),
            PaymentType::Manual => write!(f, "manual"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Completed => write!(f, "completed"),
            PaymentStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositType {
    Security,
    Pet,
    LastMonth,
}

impl std::fmt::Display for DepositType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepositType::Security => write!(f, "security"),
            DepositType::Pet => write!(f, "pet"),
            DepositType::LastMonth => write!(f, "last_month"),
        }
    }
}

// ---------------------------------------------------------------------------
// Shared data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

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
    pub role: UserRole,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub user_id: Option<String>,
    pub amount: f64,
    pub payment_type: PaymentType,
    pub description: Option<String>,
    pub stripe_payment_id: Option<String>,
    pub status: PaymentStatus,
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
    pub status: MaintenanceStatus,
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
pub struct TenantUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub email: String,
    pub password: String,
    pub name: String,
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
    pub deposit_type: DepositType,
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

// ---------------------------------------------------------------------------
// Applicants / Leads
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Applicant {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub desired_move_in: Option<String>,
    pub message: Option<String>,
    pub status: ApplicantStatus,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApplicantRequest {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub desired_move_in: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplicantRequest {
    pub status: Option<ApplicantStatus>,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Admin Reports
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueMonth {
    pub month: String, // "2026-03"
    pub total: f64,
    pub rent: f64,
    pub utility: f64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceStats {
    pub total: u32,
    pub submitted: u32,
    pub in_progress: u32,
    pub completed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewReport {
    pub total_collected: f64,
    pub total_outstanding: f64,
    pub total_payments: u32,
    pub active_tenants: u32,
    pub maintenance_stats: MaintenanceStats,
    pub monthly_revenue: Vec<RevenueMonth>,
}

// ---------------------------------------------------------------------------
// Lease Agreement
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseAgreement {
    pub id: String,
    pub tenant_id: String,
    pub document_id: Option<String>,
    pub status: String, // pending, sent, signed_by_tenant, executed, expired
    pub rent_amount: f64,
    pub lease_start: Option<String>,
    pub lease_end: Option<String>,
    pub tenant_signed_name: Option<String>,
    pub tenant_signed_at: Option<String>,
    pub landlord_signed_name: Option<String>,
    pub landlord_signed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignLeaseRequest {
    pub lease_id: String,
    pub full_legal_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLeaseRequest {
    pub tenant_id: String,
    pub rent_amount: f64,
    pub lease_start: String,
    pub lease_end: String,
}
