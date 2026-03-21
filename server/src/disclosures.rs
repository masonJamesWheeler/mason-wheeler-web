use axum::{
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use mason_wheeler_shared::*;
use uuid::Uuid;

use crate::db::get_db;
use crate::routes::extract_user;

pub fn disclosures_router() -> Router {
    Router::new()
        // Move-in checklist
        .route("/checklist", get(get_checklist).post(create_checklist))
        .route("/checklist/sign", post(sign_checklist))
        // Lead paint disclosure
        .route("/lead-paint", get(get_lead_paint).post(create_lead_paint))
        .route("/lead-paint/acknowledge", post(acknowledge_lead_paint))
        // Mold info (static content served from the app)
        .route("/mold-info", get(get_mold_info))
        // Deposit receipt
        .route("/deposit-receipt", get(get_deposit_receipt).post(create_deposit_receipt))
        // Fee-in-lieu disclosure
        .route("/fee-in-lieu", get(get_fee_in_lieu_info))
        // Landlord contact
        .route("/landlord-contact", get(get_landlord_contact).put(update_landlord_contact))
}

// ---------------------------------------------------------------------------
// Move-In Checklist — RCW 59.18.260
// ---------------------------------------------------------------------------

/// Default checklist items per room for a 2bd/1ba bungalow
fn default_checklist_rooms() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Living Room", vec![
            "Walls — paint condition", "Walls — holes or damage", "Ceiling — condition",
            "Flooring — hardwood condition", "Windows — operation and locks",
            "Window coverings — blinds/curtains", "Light fixtures", "Electrical outlets",
            "Fireplace — stone surround condition", "Fireplace — damper operation",
            "Front door — condition and lock", "Baseboards/trim",
        ]),
        ("Kitchen", vec![
            "Walls — paint and tile condition", "Ceiling — condition",
            "Flooring — tile condition", "Countertops — condition",
            "Cabinets — doors and hinges", "Sink and faucet", "Dishwasher",
            "Gas range/stove", "Range hood/vent", "Refrigerator — exterior",
            "Refrigerator — interior/shelves", "Garbage disposal",
            "Light fixtures", "Electrical outlets", "Windows — operation",
            "Subway tile backsplash — condition",
        ]),
        ("Bedroom 1 (Primary)", vec![
            "Walls — paint condition", "Walls — holes or damage", "Ceiling — condition",
            "Flooring — hardwood condition", "Windows — operation and locks",
            "Window coverings", "Closet — doors/shelves", "Light fixtures",
            "Electrical outlets", "Ceiling fan — operation",
        ]),
        ("Bedroom 2", vec![
            "Walls — paint condition", "Walls — holes or damage", "Ceiling — condition",
            "Flooring — hardwood condition", "Windows — operation and locks",
            "Window coverings", "Closet — doors/shelves", "Light fixtures",
            "Electrical outlets", "Ceiling fan — operation",
        ]),
        ("Bathroom", vec![
            "Walls — paint and tile condition", "Ceiling — condition",
            "Flooring — condition", "Bathtub/shower — condition",
            "Shower tile — condition and grout", "Toilet — operation",
            "Sink and faucet", "Vanity cabinet — condition", "Mirror — condition",
            "Towel bars/hooks", "Light fixtures", "Exhaust fan — operation",
            "Window — condition",
        ]),
        ("Den", vec![
            "Walls — paint condition", "Walls — holes or damage", "Ceiling — condition",
            "Flooring — condition", "Sliding glass door — operation and lock",
            "Windows — operation and locks", "Window coverings",
            "Light fixtures", "Electrical outlets",
        ]),
        ("Laundry Area", vec![
            "Washer — condition and operation", "Dryer — condition and operation",
            "Washer hookups — condition", "Dryer vent — condition",
            "Flooring — condition", "Light fixtures",
        ]),
        ("Exterior/Yard", vec![
            "Front yard — condition", "Walkway/path — condition",
            "Front door — paint and hardware", "Siding — condition",
            "Roof — visible condition", "Gutters — condition",
            "Backyard — fence condition", "Backyard — gate operation",
            "Fire pit area — condition", "Raised garden beds — condition",
            "Storage shed — condition and lock", "Driveway — condition",
            "Exterior lighting",
        ]),
        ("General/Systems", vec![
            "Smoke detectors — all working", "Carbon monoxide detectors — all working",
            "HVAC/heating — operation", "Water heater — condition",
            "Plumbing — no visible leaks", "Electrical panel — labeled and accessible",
            "Keys — all provided and working", "Garage/shed keys",
            "Mailbox — condition and key",
        ]),
    ]
}

async fn get_checklist(headers: HeaderMap) -> Result<Json<Option<MoveInChecklist>>, StatusCode> {
    let _user = extract_user(&headers)?;
    let db = get_db();

    let checklist = db.query_row(
        "SELECT id, property_address, tenant_name, landlord_name, move_in_date, tenant_signed, landlord_signed, created_at
         FROM move_in_checklists ORDER BY created_at DESC LIMIT 1",
        [],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, bool>(5)?,
            row.get::<_, bool>(6)?,
            row.get::<_, String>(7)?,
        )),
    );

    let (id, address, tenant, landlord, date, t_signed, l_signed, created) = match checklist {
        Ok(c) => c,
        Err(_) => return Ok(Json(None)),
    };

    let mut stmt = db.prepare(
        "SELECT id, room, item, condition, notes, photo_path, created_at
         FROM checklist_items WHERE checklist_id = ?1 ORDER BY rowid ASC"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<ChecklistItem> = stmt.query_map(rusqlite::params![id], |row| {
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

    // Group items by room
    let mut room_map: std::collections::BTreeMap<String, Vec<ChecklistItem>> = std::collections::BTreeMap::new();
    for item in items {
        room_map.entry(item.room.clone()).or_default().push(item);
    }
    let rooms: Vec<ChecklistRoom> = room_map.into_iter().map(|(name, items)| ChecklistRoom { name, items }).collect();

    Ok(Json(Some(MoveInChecklist {
        id,
        property_address: address,
        tenant_name: tenant,
        landlord_name: landlord,
        move_in_date: date,
        tenant_signed: t_signed,
        landlord_signed: l_signed,
        rooms,
        created_at: created,
    })))
}

async fn create_checklist(
    headers: HeaderMap,
    Json(body): Json<CreateChecklistRequest>,
) -> Result<Json<MoveInChecklist>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let checklist_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let db = get_db();

    db.execute(
        "INSERT INTO move_in_checklists (id, property_address, tenant_name, landlord_name, move_in_date, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            checklist_id,
            "8404 12th Ave S, Seattle, WA 98108",
            body.tenant_name,
            "Mason Wheeler",
            body.move_in_date,
            now,
        ],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If no items provided, use defaults
    let items_to_insert = if body.items.is_empty() {
        default_checklist_rooms().into_iter().flat_map(|(room, items)| {
            items.into_iter().map(move |item| CreateChecklistItem {
                room: room.to_string(),
                item: item.to_string(),
                condition: "good".to_string(),
                notes: None,
            })
        }).collect::<Vec<_>>()
    } else {
        body.items
    };

    let mut rooms_map: std::collections::BTreeMap<String, Vec<ChecklistItem>> = std::collections::BTreeMap::new();

    for ci in &items_to_insert {
        let item_id = Uuid::new_v4().to_string();
        db.execute(
            "INSERT INTO checklist_items (id, checklist_id, room, item, condition, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![item_id, checklist_id, ci.room, ci.item, ci.condition, ci.notes, now],
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        rooms_map.entry(ci.room.clone()).or_default().push(ChecklistItem {
            id: item_id,
            room: ci.room.clone(),
            item: ci.item.clone(),
            condition: ci.condition.clone(),
            notes: ci.notes.clone(),
            photo_path: None,
            created_at: now.clone(),
        });
    }

    let rooms: Vec<ChecklistRoom> = rooms_map.into_iter().map(|(name, items)| ChecklistRoom { name, items }).collect();

    Ok(Json(MoveInChecklist {
        id: checklist_id,
        property_address: "8404 12th Ave S, Seattle, WA 98108".to_string(),
        tenant_name: body.tenant_name,
        landlord_name: "Mason Wheeler".to_string(),
        move_in_date: body.move_in_date,
        tenant_signed: false,
        landlord_signed: false,
        rooms,
        created_at: now,
    }))
}

#[derive(serde::Deserialize)]
struct SignRequest {
    role: String, // "tenant" or "landlord"
}

async fn sign_checklist(
    headers: HeaderMap,
    Json(body): Json<SignRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = extract_user(&headers)?;
    let db = get_db();

    let column = match body.role.as_str() {
        "tenant" if user.role == "tenant" => "tenant_signed",
        "landlord" if user.role == "landlord" => "landlord_signed",
        _ => return Err(StatusCode::FORBIDDEN),
    };

    db.execute(
        &format!("UPDATE move_in_checklists SET {} = 1 WHERE id = (SELECT id FROM move_in_checklists ORDER BY created_at DESC LIMIT 1)", column),
        [],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "signed": true, "role": body.role })))
}

// ---------------------------------------------------------------------------
// Lead Paint Disclosure — Federal law, pre-1978
// ---------------------------------------------------------------------------

async fn get_lead_paint(headers: HeaderMap) -> Result<Json<Option<LeadPaintDisclosure>>, StatusCode> {
    let _user = extract_user(&headers)?;
    let db = get_db();

    let result = db.query_row(
        "SELECT property_address, year_built, known_lead_paint, known_hazards_description,
                records_available, records_description, tenant_name, landlord_name,
                tenant_acknowledged, landlord_signed, date_signed
         FROM lead_paint_disclosures ORDER BY created_at DESC LIMIT 1",
        [],
        |row| Ok(LeadPaintDisclosure {
            property_address: row.get(0)?,
            year_built: row.get(1)?,
            known_lead_paint: row.get(2)?,
            known_hazards_description: row.get(3)?,
            records_available: row.get(4)?,
            records_description: row.get(5)?,
            tenant_name: row.get(6)?,
            landlord_name: row.get(7)?,
            tenant_acknowledged: row.get(8)?,
            landlord_signed: row.get(9)?,
            date: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
        }),
    );

    match result {
        Ok(d) => Ok(Json(Some(d))),
        Err(_) => Ok(Json(None)),
    }
}

async fn create_lead_paint(
    headers: HeaderMap,
    Json(body): Json<LeadPaintDisclosure>,
) -> Result<Json<LeadPaintDisclosure>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let db = get_db();

    db.execute(
        "INSERT INTO lead_paint_disclosures (id, property_address, year_built, known_lead_paint, known_hazards_description, records_available, records_description, tenant_name, landlord_name, landlord_signed, date_signed, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            id, body.property_address, body.year_built, body.known_lead_paint,
            body.known_hazards_description, body.records_available, body.records_description,
            body.tenant_name, body.landlord_name, body.landlord_signed, body.date, now,
        ],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(body))
}

async fn acknowledge_lead_paint(headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "tenant" {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = get_db();
    db.execute(
        "UPDATE lead_paint_disclosures SET tenant_acknowledged = 1 WHERE id = (SELECT id FROM lead_paint_disclosures ORDER BY created_at DESC LIMIT 1)",
        [],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "acknowledged": true })))
}

// ---------------------------------------------------------------------------
// Mold Information — RCW 59.18.060(13)
// ---------------------------------------------------------------------------

/// Returns WA DOH-approved mold disclosure content
async fn get_mold_info(headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user = extract_user(&headers)?;

    Ok(Json(serde_json::json!({
        "title": "Mold Information for Tenants",
        "source": "Washington State Department of Health & EPA",
        "required_by": "RCW 59.18.060(13)",
        "sections": [
            {
                "heading": "What is Mold?",
                "content": "Molds are fungi that can be found both indoors and outdoors. They grow best in warm, damp, and humid conditions. Mold spreads by making spores which can survive dry, harsh environmental conditions."
            },
            {
                "heading": "Health Effects of Indoor Mold",
                "content": "Exposure to mold can cause health effects in some people. Mold spores can cause allergic reactions, including sneezing, runny nose, red eyes, and skin rash. People with asthma who are allergic to mold may experience asthma attacks. People with weakened immune systems and chronic lung diseases may get serious infections in their lungs from mold."
            },
            {
                "heading": "How to Prevent Mold Growth",
                "content": "Keep humidity levels between 30-50%. Use exhaust fans or open windows when showering, cooking, or using the dishwasher. Fix water leaks promptly. Ensure good air circulation — do not block air vents with furniture. Clean and dry any damp or wet building materials and furnishings within 24-48 hours. Clean bathrooms regularly with mold-killing products."
            },
            {
                "heading": "Ventilation Tips",
                "content": "Run bathroom exhaust fans during and for 30 minutes after showers. Use kitchen exhaust fan when cooking. Open windows when weather permits to increase air circulation. Do not cover or block heating/cooling vents. Use a dehumidifier if needed, especially in the den area."
            },
            {
                "heading": "Signs of Mold",
                "content": "Visible mold growth (may look like spots of various colors). Musty, earthy smell. Water stains or discoloration on walls, ceilings, or floors. Peeling or bubbling paint or wallpaper. Warped wood."
            },
            {
                "heading": "What to Do If You Find Mold",
                "content": "Report any mold growth or water leaks to your landlord immediately. Small areas of mold (less than 10 square feet) on hard surfaces can be cleaned with soap and water. Do NOT use bleach. For larger areas, professional remediation is needed. Document the mold with photos and written descriptions."
            },
            {
                "heading": "Tenant Responsibilities",
                "content": "Use ventilation fans when bathing and cooking. Report water leaks, drips, or moisture problems promptly. Clean and maintain the dwelling to prevent mold growth. Do not block air vents. Notify landlord immediately if you see mold growth."
            },
            {
                "heading": "Landlord Responsibilities",
                "content": "Maintain the dwelling in a condition that does not contribute to mold growth. Fix water leaks and moisture problems promptly. Ensure proper ventilation systems are installed and operational. Address reported mold issues in a timely manner."
            }
        ],
        "references": [
            "Washington State Department of Health — Renters, Landlords, and Mold",
            "EPA — A Brief Guide to Mold, Moisture, and Your Home",
            "RCW 59.18.060(13) — Landlord duty to provide mold information"
        ]
    })))
}

// ---------------------------------------------------------------------------
// Deposit Receipt — RCW 59.18.260, 59.18.270
// ---------------------------------------------------------------------------

async fn get_deposit_receipt(headers: HeaderMap) -> Result<Json<Option<DepositReceipt>>, StatusCode> {
    let _user = extract_user(&headers)?;
    let db = get_db();

    let result = db.query_row(
        "SELECT tenant_name, property_address, deposit_amount, deposit_type, depository_name, depository_address, date_received, landlord_name
         FROM deposit_receipts ORDER BY created_at DESC LIMIT 1",
        [],
        |row| Ok(DepositReceipt {
            tenant_name: row.get(0)?,
            property_address: row.get(1)?,
            deposit_amount: row.get(2)?,
            deposit_type: row.get(3)?,
            depository_name: row.get(4)?,
            depository_address: row.get(5)?,
            date_received: row.get(6)?,
            landlord_name: row.get(7)?,
        }),
    );

    match result {
        Ok(r) => Ok(Json(Some(r))),
        Err(_) => Ok(Json(None)),
    }
}

async fn create_deposit_receipt(
    headers: HeaderMap,
    Json(body): Json<DepositReceipt>,
) -> Result<Json<DepositReceipt>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let db = get_db();

    db.execute(
        "INSERT INTO deposit_receipts (id, tenant_name, property_address, deposit_amount, deposit_type, depository_name, depository_address, date_received, landlord_name, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            id, body.tenant_name, body.property_address, body.deposit_amount,
            body.deposit_type, body.depository_name, body.depository_address,
            body.date_received, body.landlord_name, now,
        ],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(body))
}

// ---------------------------------------------------------------------------
// Fee-in-Lieu Disclosure — RCW 59.18.670
// ---------------------------------------------------------------------------

async fn get_fee_in_lieu_info(headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    let _user = extract_user(&headers)?;

    Ok(Json(serde_json::json!({
        "title": "Fee in Lieu of Security Deposit Disclosure",
        "required_by": "RCW 59.18.670",
        "disclosure": {
            "description": "Washington State law requires landlords to disclose whether a fee-in-lieu-of-deposit option is available. This disclosure must accompany any lease and renewal.",
            "option_available": false,
            "explanation": "At this time, the landlord does not offer a fee-in-lieu-of-security-deposit option for this property. A traditional security deposit is required.",
            "tenant_rights": [
                "You have the right to pay the security deposit in installments per RCW 59.18.610",
                "Installment payments may be spread over the first 6 months of the lease",
                "The landlord must disclose this option with every new lease and renewal",
                "If a fee-in-lieu option were offered, it would be nonrefundable and would not protect you from damage claims"
            ],
            "security_deposit_info": {
                "amount": "To be determined",
                "installment_option": true,
                "installment_period": "Up to 6 months from lease commencement"
            }
        }
    })))
}

// ---------------------------------------------------------------------------
// Landlord Contact Information
// ---------------------------------------------------------------------------

async fn get_landlord_contact(headers: HeaderMap) -> Result<Json<LandlordContactInfo>, StatusCode> {
    let _user = extract_user(&headers)?;
    let db = get_db();

    let result = db.query_row(
        "SELECT name, mailing_address, phone, email, emergency_contact, emergency_phone FROM landlord_contact WHERE id = 'main'",
        [],
        |row| Ok(LandlordContactInfo {
            name: row.get(0)?,
            mailing_address: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            emergency_contact: row.get(4)?,
            emergency_phone: row.get(5)?,
        }),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn update_landlord_contact(
    headers: HeaderMap,
    Json(body): Json<LandlordContactInfo>,
) -> Result<Json<LandlordContactInfo>, StatusCode> {
    let user = extract_user(&headers)?;
    if user.role != "landlord" {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = get_db();
    db.execute(
        "UPDATE landlord_contact SET name = ?1, mailing_address = ?2, phone = ?3, email = ?4, emergency_contact = ?5, emergency_phone = ?6, updated_at = ?7 WHERE id = 'main'",
        rusqlite::params![
            body.name, body.mailing_address, body.phone, body.email,
            body.emergency_contact, body.emergency_phone,
            chrono::Utc::now().to_rfc3339(),
        ],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(body))
}
