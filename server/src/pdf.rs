use genpdf::elements::{Break, LinearLayout, Paragraph, TableLayout};
use genpdf::style::{Style, StyledString};
use genpdf::{Document, SimplePageDecorator};

const PROPERTY_ADDRESS: &str = "8404 12th Ave S, Seattle, WA 98108";

fn new_doc(title: &str) -> Result<Document, anyhow::Error> {
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
