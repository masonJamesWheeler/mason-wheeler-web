use genpdf::elements::{Break, LinearLayout, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::{Style, StyledString};
use genpdf::{Document, SimplePageDecorator};
use std::sync::OnceLock;

const PROPERTY_ADDRESS: &str = "8404 12th Ave S, Seattle, WA 98108";

static CACHED_FONTS: OnceLock<FontFamily<FontData>> = OnceLock::new();

fn get_font_family() -> Result<FontFamily<FontData>, anyhow::Error> {
    if let Some(fonts) = CACHED_FONTS.get() {
        return Ok(fonts.clone());
    }

    let font_family =
        genpdf::fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSans", None)
            .or_else(|_| {
                genpdf::fonts::from_files(
                    "/usr/local/share/fonts/liberation",
                    "LiberationSans",
                    None,
                )
            })
            .or_else(|_| {
                genpdf::fonts::from_files(
                    "/System/Library/Fonts/Supplemental",
                    "Arial",
                    None,
                )
            })
            .or_else(|_| {
                genpdf::fonts::from_files(
                    "/opt/homebrew/share/fonts/liberation",
                    "LiberationSans",
                    None,
                )
            })?;

    // Store in cache (ignore if another thread beat us)
    let _ = CACHED_FONTS.set(font_family.clone());
    Ok(font_family)
}

fn new_doc(title: &str) -> Result<Document, anyhow::Error> {
    let font_family = get_font_family()?;

    let mut doc = Document::new(font_family);
    doc.set_title(title);
    doc.set_font_size(10);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    Ok(doc)
}

fn title_style() -> Style {
    Style::new().bold().with_font_size(18)
}

fn heading_style() -> Style {
    Style::new().bold().with_font_size(12)
}

fn body_style() -> Style {
    Style::new().with_font_size(10)
}

fn bold_small_style() -> Style {
    Style::new().bold().with_font_size(9)
}

fn title_paragraph(text: &str) -> Paragraph {
    Paragraph::new(StyledString::new(text, title_style()))
}

fn heading_paragraph(text: &str) -> Paragraph {
    Paragraph::new(StyledString::new(text, heading_style()))
}

fn body_paragraph(text: &str) -> Paragraph {
    Paragraph::new(StyledString::new(text, body_style()))
}

fn signature_block() -> LinearLayout {
    let mut layout = LinearLayout::vertical();
    layout.push(Break::new(2.0));
    layout.push(body_paragraph(
        "________________________________________     ________________",
    ));
    layout.push(body_paragraph("Tenant Signature                                            Date"));
    layout.push(Break::new(1.5));
    layout.push(body_paragraph(
        "________________________________________     ________________",
    ));
    layout.push(body_paragraph("Landlord Signature                                         Date"));
    layout
}

// ---------------------------------------------------------------------------
// Move-In Checklist PDF
// ---------------------------------------------------------------------------

pub fn generate_move_in_checklist_pdf(checklist_data: &str) -> Result<Vec<u8>, anyhow::Error> {
    let checklist: mason_wheeler_shared::MoveInChecklist = serde_json::from_str(checklist_data)?;

    let mut doc = new_doc("Move-In / Move-Out Checklist")?;

    doc.push(title_paragraph("Move-In / Move-Out Checklist"));
    doc.push(Break::new(0.5));
    doc.push(body_paragraph(&format!(
        "Property: {}",
        checklist.property_address
    )));
    doc.push(body_paragraph(&format!(
        "Tenant: {}",
        checklist.tenant_name
    )));
    doc.push(body_paragraph(&format!(
        "Landlord: {}",
        checklist.landlord_name
    )));
    doc.push(body_paragraph(&format!(
        "Move-In Date: {}",
        checklist.move_in_date
    )));
    doc.push(body_paragraph("Per RCW 59.18.260"));
    doc.push(Break::new(1.0));

    for room in &checklist.rooms {
        doc.push(heading_paragraph(&room.name));

        let mut table = TableLayout::new(vec![3, 1, 2]);
        table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

        // Header row
        table
            .row()
            .element(Paragraph::new(StyledString::new("Item", bold_small_style())))
            .element(Paragraph::new(StyledString::new("Condition", bold_small_style())))
            .element(Paragraph::new(StyledString::new("Notes", bold_small_style())))
            .push()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        for item in &room.items {
            let notes = item.notes.as_deref().unwrap_or("");
            table
                .row()
                .element(body_paragraph(&item.item))
                .element(body_paragraph(&item.condition))
                .element(body_paragraph(notes))
                .push()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        doc.push(table);
        doc.push(Break::new(0.5));
    }

    doc.push(signature_block());

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Lead Paint Disclosure PDF
// ---------------------------------------------------------------------------

pub fn generate_lead_paint_disclosure_pdf(
    property_address: &str,
    year_built: i32,
    landlord_name: &str,
    tenant_name: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = new_doc("Disclosure of Information on Lead-Based Paint")?;

    doc.push(title_paragraph(
        "Disclosure of Information on Lead-Based Paint and/or Lead-Based Paint Hazards",
    ));
    doc.push(Break::new(0.5));
    doc.push(body_paragraph("(Required by Federal law for housing built before 1978)"));
    doc.push(Break::new(1.0));

    doc.push(body_paragraph(&format!("Property Address: {}", property_address)));
    doc.push(body_paragraph(&format!("Year Built: {}", year_built)));
    doc.push(Break::new(0.5));

    doc.push(heading_paragraph("Lead Warning Statement"));
    doc.push(body_paragraph(
        "Housing built before 1978 may contain lead-based paint. Lead from paint, paint chips, \
         and dust can pose health hazards if not managed properly. Lead exposure is especially \
         harmful to young children and pregnant women. Before renting pre-1978 housing, lessors \
         must disclose the presence of known lead-based paint and/or lead-based paint hazards in \
         the dwelling. Lessees must also receive a federally approved pamphlet on lead poisoning \
         prevention.",
    ));
    doc.push(Break::new(0.5));

    doc.push(heading_paragraph("Lessor's Disclosure"));
    doc.push(body_paragraph(
        "(a) Presence of lead-based paint and/or lead-based paint hazards (check one):",
    ));
    doc.push(body_paragraph(
        "  [ ] Known lead-based paint and/or lead-based paint hazards are present in the housing.",
    ));
    doc.push(body_paragraph(
        "  [X] Lessor has no knowledge of lead-based paint and/or lead-based paint hazards in the housing.",
    ));
    doc.push(Break::new(0.3));
    doc.push(body_paragraph(
        "(b) Records and reports available to the lessor (check one):",
    ));
    doc.push(body_paragraph(
        "  [ ] Lessor has provided the lessee with all available records and reports pertaining to \
         lead-based paint and/or lead-based paint hazards in the housing.",
    ));
    doc.push(body_paragraph(
        "  [X] Lessor has no reports or records pertaining to lead-based paint and/or lead-based \
         paint hazards in the housing.",
    ));
    doc.push(Break::new(0.5));

    doc.push(heading_paragraph("Lessee's Acknowledgment"));
    doc.push(body_paragraph(
        "(c) The lessee has received copies of all information listed above.",
    ));
    doc.push(body_paragraph(
        "(d) The lessee has received the pamphlet \"Protect Your Family from Lead in Your Home.\"",
    ));
    doc.push(Break::new(0.5));

    doc.push(heading_paragraph("Certification of Accuracy"));
    doc.push(body_paragraph(
        "The following parties have reviewed the information above and certify, to the best of \
         their knowledge, that the information they have provided is true and accurate.",
    ));
    doc.push(Break::new(1.0));

    doc.push(body_paragraph(&format!("Lessor: {}", landlord_name)));
    doc.push(body_paragraph(
        "________________________________________     ________________",
    ));
    doc.push(body_paragraph("Lessor Signature                                            Date"));
    doc.push(Break::new(1.5));

    doc.push(body_paragraph(&format!("Lessee: {}", tenant_name)));
    doc.push(body_paragraph(
        "________________________________________     ________________",
    ));
    doc.push(body_paragraph("Lessee Signature                                            Date"));

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Deposit Receipt PDF
// ---------------------------------------------------------------------------

pub fn generate_deposit_receipt_pdf(
    tenant_name: &str,
    amount: f64,
    deposit_type: &str,
    depository: &str,
    date: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = new_doc("Security Deposit Receipt")?;

    doc.push(title_paragraph("Security Deposit Receipt"));
    doc.push(Break::new(0.5));
    doc.push(body_paragraph("Per RCW 59.18.260 and RCW 59.18.270"));
    doc.push(Break::new(1.0));

    doc.push(body_paragraph(&format!("Property Address: {}", PROPERTY_ADDRESS)));
    doc.push(body_paragraph(&format!("Tenant Name: {}", tenant_name)));
    doc.push(body_paragraph(&format!("Date Received: {}", date)));
    doc.push(Break::new(0.5));

    let type_label = match deposit_type {
        "security" => "Security Deposit",
        "pet" => "Pet Deposit",
        "last_month" => "Last Month's Rent",
        other => other,
    };

    doc.push(heading_paragraph("Deposit Details"));
    doc.push(body_paragraph(&format!("Deposit Type: {}", type_label)));
    doc.push(body_paragraph(&format!("Amount: ${:.2}", amount)));
    doc.push(Break::new(0.5));

    doc.push(heading_paragraph("Depository Information"));
    doc.push(body_paragraph(&format!("Depository: {}", depository)));
    doc.push(Break::new(0.5));

    doc.push(heading_paragraph("Terms"));
    doc.push(body_paragraph(
        "This deposit shall be held in accordance with Washington State law (RCW 59.18.260). \
         The deposit, or the portion thereof not used to cover unpaid rent or damages beyond \
         normal wear and tear, shall be returned within 21 days after the tenancy has ended \
         and the tenant has vacated the premises. A full and specific statement of the basis \
         for retaining any portion of the deposit shall be provided with any refund.",
    ));
    doc.push(Break::new(1.0));

    doc.push(signature_block());

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Payment Receipt PDF
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Residential Lease Agreement PDF
// ---------------------------------------------------------------------------

pub struct LeaseParams {
    pub landlord_name: String,
    pub landlord_address: String,
    pub tenant_names: Vec<String>,
    pub rent_amount: f64,
    pub security_deposit: f64,
    pub pet_deposit: f64,
    pub lease_start: String,
    pub lease_end: String,
    pub move_in_date: String,
    pub max_occupants: u32,
    pub pets_description: String,
    pub rrio_number: String,
    pub depository_name: String,
    pub depository_address: String,
}

pub fn generate_lease_pdf(p: &LeaseParams) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = new_doc("Residential Lease Agreement")?;
    let tenant_list = p.tenant_names.join(", ");
    let total_deposit = p.security_deposit + p.pet_deposit;

    // ── Title ──
    doc.push(title_paragraph("RESIDENTIAL LEASE AGREEMENT"));
    doc.push(Break::new(0.3));
    doc.push(body_paragraph("State of Washington — City of Seattle"));
    doc.push(Break::new(1.0));

    // ── 1. Parties ──
    doc.push(heading_paragraph("1. PARTIES"));
    doc.push(body_paragraph(&format!(
        "This Residential Lease Agreement (\"Lease\") is entered into between {} (\"Landlord\"), \
         with a mailing address of {}, and {} (\"Tenant\"), collectively referred to as the \"Parties.\"",
        p.landlord_name, p.landlord_address, tenant_list
    )));
    doc.push(Break::new(0.5));

    // ── 2. Property ──
    doc.push(heading_paragraph("2. PROPERTY"));
    doc.push(body_paragraph(&format!(
        "Landlord agrees to rent to Tenant the property located at {} (\"Premises\"), \
         a single-family residence, for use as a private dwelling only.",
        PROPERTY_ADDRESS
    )));
    doc.push(Break::new(0.5));

    // ── 3. Term ──
    doc.push(heading_paragraph("3. TERM"));
    doc.push(body_paragraph(&format!(
        "This Lease begins on {} and ends on {}, for a total term of twelve (12) months. \
         Upon expiration, this Lease shall automatically convert to a month-to-month tenancy \
         under the same terms unless either party provides written notice as required by law.",
        p.lease_start, p.lease_end
    )));
    doc.push(Break::new(0.5));

    // ── 4. Rent ──
    doc.push(heading_paragraph("4. RENT"));
    doc.push(body_paragraph(&format!(
        "Tenant shall pay Landlord a monthly rent of ${:.2}. Rent is due on the first (1st) day \
         of each month. Rent may be paid electronically via the property management portal at \
         properties.mason-wheeler.com, or by check or money order delivered to the Landlord's \
         mailing address.",
        p.rent_amount
    )));
    doc.push(Break::new(0.3));
    doc.push(body_paragraph(
        "A five (5) day grace period applies to all electronic payments per Washington State law \
         (RCW 59.18). No late fee shall be assessed during this grace period. After the grace period, \
         a late fee of $75.00 shall be assessed. Landlord shall apply all payments to rent first \
         before applying to other charges (RCW 59.18.283)."
    ));
    doc.push(Break::new(0.5));

    // ── 5. Security Deposit ──
    doc.push(heading_paragraph("5. SECURITY DEPOSIT"));
    doc.push(body_paragraph(&format!(
        "Upon execution of this Lease, Tenant shall pay a security deposit of ${:.2}{}. \
         The total of all deposits and nonrefundable fees shall not exceed one month's rent \
         per Seattle Municipal Code. Tenant may request to pay the deposit in installments.",
        p.security_deposit,
        if p.pet_deposit > 0.0 {
            format!(" and a pet deposit of ${:.2} (total: ${:.2})", p.pet_deposit, total_deposit)
        } else {
            String::new()
        }
    )));
    doc.push(Break::new(0.3));
    doc.push(body_paragraph(&format!(
        "The deposit shall be held at {} located at {}. \
         The deposit, or portion thereof, shall be returned within twenty-one (21) days after \
         the tenancy ends and the Tenant has vacated, along with a full and specific statement \
         of the basis for any deductions (RCW 59.18.280).",
        p.depository_name, p.depository_address
    )));
    doc.push(Break::new(0.3));
    doc.push(body_paragraph(
        "FEE-IN-LIEU OPTION: Per RCW 59.18.610, Tenant has the option to pay a nonrefundable \
         fee in lieu of a security deposit. Tenant may contact Landlord for details on this option."
    ));
    doc.push(Break::new(0.5));

    // ── 6. Move-In Checklist ──
    doc.push(heading_paragraph("6. MOVE-IN CONDITION CHECKLIST"));
    doc.push(body_paragraph(
        "Prior to move-in, Landlord and Tenant shall jointly complete a written checklist \
         describing the condition and cleanliness of the Premises, including all furnishings \
         and appliances, as required by RCW 59.18.260. This checklist is incorporated by \
         reference into this Lease."
    ));
    doc.push(Break::new(0.5));

    // ── 7. Occupancy ──
    doc.push(heading_paragraph("7. OCCUPANCY"));
    doc.push(body_paragraph(&format!(
        "The Premises shall be occupied only by the following named Tenants: {}. \
         Maximum occupancy is {} persons. Guests staying longer than fourteen (14) consecutive \
         days must be approved in writing by Landlord. No subletting or assignment of this Lease, \
         including short-term rental listings (e.g., Airbnb), is permitted without prior written \
         consent of Landlord.",
        tenant_list, p.max_occupants
    )));
    doc.push(Break::new(0.5));

    // ── 8. Pets ──
    doc.push(heading_paragraph("8. PETS"));
    if p.pets_description.is_empty() {
        doc.push(body_paragraph("No pets are authorized under this Lease."));
    } else {
        doc.push(body_paragraph(&format!(
            "The following pets are authorized: {}. Tenant is responsible for all damage caused \
             by pets beyond normal wear and tear. A maximum of two (2) pets is permitted unless \
             otherwise agreed in writing. Tenant shall comply with all local animal control laws.",
            p.pets_description
        )));
    }
    doc.push(Break::new(0.5));

    // ── 9. Maintenance and Repairs ──
    doc.push(heading_paragraph("9. MAINTENANCE AND REPAIRS"));
    doc.push(body_paragraph(
        "Landlord shall maintain the Premises in compliance with all applicable building and \
         housing codes (RCW 59.18.060). Landlord shall commence remedial action within: \
         24 hours for loss of hot/cold water, heat, electricity, or imminent safety hazards; \
         72 hours for loss of refrigerator, range/oven, or major plumbing fixtures; and \
         10 days for all other repairs."
    ));
    doc.push(Break::new(0.3));
    doc.push(body_paragraph(
        "Tenant shall maintain the Premises in a clean and sanitary condition, properly dispose \
         of garbage, and promptly notify Landlord of any maintenance issues within 48 hours of \
         discovery. Maintenance requests shall be submitted through the property management \
         portal at properties.mason-wheeler.com."
    ));
    doc.push(Break::new(0.5));

    // ── 10. Landlord Access ──
    doc.push(heading_paragraph("10. LANDLORD ACCESS"));
    doc.push(body_paragraph(
        "Landlord may enter the Premises with at least two (2) days' written notice for repairs \
         and maintenance, and one (1) day's notice for showing the property to prospective tenants \
         or buyers. In case of emergency, Landlord may enter without notice. Entry shall be at \
         reasonable times (RCW 59.18.150)."
    ));
    doc.push(Break::new(0.5));

    // ── 11. Utilities ──
    doc.push(heading_paragraph("11. UTILITIES"));
    doc.push(body_paragraph(
        "Tenant shall be responsible for all utilities including electricity, gas, water/sewer, \
         garbage, and internet/cable unless otherwise agreed. Landlord may pass through utility \
         costs with documentation of the actual charges. No markup shall be applied to utility \
         pass-through charges."
    ));
    doc.push(Break::new(0.5));

    // ── 12. Renter's Insurance ──
    doc.push(heading_paragraph("12. RENTER'S INSURANCE"));
    doc.push(body_paragraph(
        "Tenant is required to maintain renter's insurance throughout the term of this Lease \
         with a minimum liability coverage of $100,000. Tenant shall provide proof of insurance \
         to Landlord prior to move-in and upon renewal. Landlord's insurance does not cover \
         Tenant's personal property."
    ));
    doc.push(Break::new(0.5));

    // ── 13. Prohibited Activities ──
    doc.push(heading_paragraph("13. PROHIBITED ACTIVITIES"));
    doc.push(body_paragraph(
        "The following are prohibited on the Premises: (a) smoking or vaping indoors; \
         (b) illegal activity of any kind; (c) use of space heaters or unattended candles; \
         (d) removal of smoke detector or CO detector batteries; (e) any activity that would \
         void the Landlord's insurance policy or violate local ordinances."
    ));
    doc.push(Break::new(0.5));

    // ── 14. Rent Increases ──
    doc.push(heading_paragraph("14. RENT INCREASES"));
    doc.push(body_paragraph(
        "Rent shall not be increased during the initial Lease term. For any subsequent \
         month-to-month tenancy, Landlord shall provide at least 180 days' written notice \
         before any rent increase, per Seattle Municipal Code. Annual rent increases shall \
         not exceed the maximum allowed under Washington State law (HB 1217: 7% + CPI, \
         capped at 10%)."
    ));
    doc.push(Break::new(0.5));

    // ── 15. Termination ──
    doc.push(heading_paragraph("15. TERMINATION AND JUST CAUSE"));
    doc.push(body_paragraph(
        "This Lease may only be terminated for just cause as defined by Seattle's Just Cause \
         Eviction Ordinance (SMC 22.206.160). Landlord must provide written notice stating \
         the specific just cause and supporting facts. For nonpayment of rent, Landlord shall \
         provide a 14-day pay-or-vacate notice per RCW 59.18.057. For month-to-month tenancy, \
         Landlord shall provide at least 20 days' written notice to terminate."
    ));
    doc.push(Break::new(0.5));

    // ── 16. Lead Paint ──
    doc.push(heading_paragraph("16. LEAD-BASED PAINT DISCLOSURE"));
    doc.push(body_paragraph(
        "The Premises were built before 1978. Tenant acknowledges receipt of the EPA pamphlet \
         \"Protect Your Family from Lead in Your Home\" and the Lead-Based Paint Disclosure \
         form, which is incorporated by reference into this Lease."
    ));
    doc.push(Break::new(0.5));

    // ── 17. Mold ──
    doc.push(heading_paragraph("17. MOLD DISCLOSURE"));
    doc.push(body_paragraph(
        "Tenant acknowledges receipt of written information about the health hazards associated \
         with exposure to indoor mold, including how to prevent mold growth, as required by \
         RCW 59.18.060(13)."
    ));
    doc.push(Break::new(0.5));

    // ── 18. RRIO ──
    doc.push(heading_paragraph("18. RENTAL REGISTRATION"));
    doc.push(body_paragraph(&format!(
        "This property is registered under Seattle's Rental Registration & Inspection Ordinance \
         (RRIO). Registration number: {}.",
        if p.rrio_number.is_empty() { "Pending" } else { &p.rrio_number }
    )));
    doc.push(Break::new(0.5));

    // ── 19. Landlord Contact ──
    doc.push(heading_paragraph("19. LANDLORD CONTACT INFORMATION"));
    doc.push(body_paragraph(&format!("Name: {}", p.landlord_name)));
    doc.push(body_paragraph(&format!("Mailing Address: {}", p.landlord_address)));
    doc.push(body_paragraph("Email: masonwheeler@fieldflow.us"));
    doc.push(Break::new(0.5));

    // ── 20. Governing Law ──
    doc.push(heading_paragraph("20. GOVERNING LAW"));
    doc.push(body_paragraph(
        "This Lease shall be governed by the laws of the State of Washington, including the \
         Residential Landlord-Tenant Act (RCW 59.18), Seattle Municipal Code, and all applicable \
         local ordinances. Any provision of this Lease that conflicts with applicable law shall \
         be void and the law shall govern."
    ));
    doc.push(Break::new(0.5));

    // ── 21. Entire Agreement ──
    doc.push(heading_paragraph("21. ENTIRE AGREEMENT"));
    doc.push(body_paragraph(
        "This Lease, together with the Move-In Condition Checklist, Lead-Based Paint Disclosure, \
         and all other referenced documents, constitutes the entire agreement between the Parties. \
         No oral agreements or representations shall be binding. Any modifications must be in \
         writing and signed by both Parties."
    ));
    doc.push(Break::new(1.0));

    // ── Signatures ──
    doc.push(heading_paragraph("SIGNATURES"));
    doc.push(Break::new(0.5));
    doc.push(body_paragraph(
        "By signing below, the Parties acknowledge that they have read and agree to all terms \
         and conditions of this Lease."
    ));
    doc.push(Break::new(1.0));

    doc.push(body_paragraph(&format!("Landlord: {}", p.landlord_name)));
    doc.push(body_paragraph(
        "________________________________________     ________________",
    ));
    doc.push(body_paragraph("Landlord Signature                                         Date"));
    doc.push(Break::new(1.5));

    for name in &p.tenant_names {
        doc.push(body_paragraph(&format!("Tenant: {}", name)));
        doc.push(body_paragraph(
            "________________________________________     ________________",
        ));
        doc.push(body_paragraph("Tenant Signature                                            Date"));
        doc.push(Break::new(1.5));
    }

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Payment Receipt PDF
// ---------------------------------------------------------------------------

pub fn generate_payment_receipt_pdf(
    tenant_name: &str,
    amount: f64,
    payment_type: &str,
    date: &str,
    description: &str,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut doc = new_doc("Payment Receipt")?;

    doc.push(title_paragraph("Payment Receipt"));
    doc.push(Break::new(0.5));
    doc.push(body_paragraph(&format!("Property Address: {}", PROPERTY_ADDRESS)));
    doc.push(Break::new(1.0));

    doc.push(body_paragraph(&format!("Tenant: {}", tenant_name)));
    doc.push(body_paragraph(&format!("Date: {}", date)));
    doc.push(Break::new(0.5));

    let type_label = match payment_type {
        "rent" => "Monthly Rent",
        "utility" => "Utility Payment",
        "deposit" => "Deposit",
        "late_fee" => "Late Fee",
        other => other,
    };

    doc.push(heading_paragraph("Payment Details"));
    doc.push(body_paragraph(&format!("Payment Type: {}", type_label)));
    doc.push(body_paragraph(&format!("Amount: ${:.2}", amount)));
    if !description.is_empty() {
        doc.push(body_paragraph(&format!("Description: {}", description)));
    }
    doc.push(Break::new(0.5));

    doc.push(body_paragraph(
        "This receipt confirms that the above payment has been received.",
    ));
    doc.push(Break::new(1.0));

    doc.push(body_paragraph(
        "________________________________________     ________________",
    ));
    doc.push(body_paragraph("Landlord Signature                                         Date"));

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}
