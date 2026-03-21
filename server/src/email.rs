use anyhow::{Context, Result};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Cached SMTP transport — built once and reused across all send_email calls
// ---------------------------------------------------------------------------

static SMTP_TRANSPORT: OnceLock<AsyncSmtpTransport<Tokio1Executor>> = OnceLock::new();

fn get_smtp_transport() -> Result<&'static AsyncSmtpTransport<Tokio1Executor>> {
    if let Some(t) = SMTP_TRANSPORT.get() {
        return Ok(t);
    }

    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);
    let smtp_user =
        std::env::var("SMTP_USER").context("SMTP_USER must be set to initialize SMTP transport")?;
    let smtp_password =
        std::env::var("SMTP_PASSWORD").context("SMTP_PASSWORD must be set when SMTP_USER is set")?;

    let creds = Credentials::new(smtp_user, smtp_password);

    let transport: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .context("failed to create SMTP transport")?
            .port(smtp_port)
            .credentials(creds)
            .build();

    // Ignore if another thread raced us
    let _ = SMTP_TRANSPORT.set(transport);
    Ok(SMTP_TRANSPORT.get().unwrap())
}

/// Send an email via SMTP. If SMTP_USER is not configured, the send is
/// skipped with a warning so that development environments work without
/// an SMTP server.
pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<()> {
    let smtp_from =
        std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@mason-wheeler.com".to_string());

    // If SMTP_USER is not set, skip gracefully for dev environments
    if std::env::var("SMTP_USER").is_err() {
        tracing::warn!(
            "SMTP_USER not set — skipping email to {to} (subject: {subject})"
        );
        return Ok(());
    }

    let mailer = get_smtp_transport()?;

    let email = Message::builder()
        .from(smtp_from.parse().context("invalid SMTP_FROM address")?)
        .to(to.parse().context("invalid recipient address")?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .context("failed to build email message")?;

    mailer
        .send(email)
        .await
        .context("failed to send email")?;

    tracing::info!("Email sent to {to} — subject: {subject}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Convenience helpers
// ---------------------------------------------------------------------------

pub async fn send_payment_confirmation(to: &str, amount: f64, date: &str) -> Result<()> {
    let subject = "Payment Confirmation";
    let body = format!(
        "Your rent payment of ${amount:.2} has been received on {date}.\n\n\
         Thank you for your prompt payment.\n\n\
         — Mason Wheeler Properties"
    );
    send_email(to, subject, &body).await
}

pub async fn send_maintenance_update(to: &str, title: &str, status: &str) -> Result<()> {
    let subject = format!("Maintenance Update: {title}");
    let body = format!(
        "Your maintenance request '{title}' has been updated to: {status}.\n\n\
         If you have any questions, please contact us.\n\n\
         — Mason Wheeler Properties"
    );
    send_email(to, &subject, &body).await
}

pub async fn send_late_rent_warning(to: &str, amount: f64, days_late: i32) -> Result<()> {
    let subject = "Rent Past Due Notice";
    let body = format!(
        "Your rent of ${amount:.2} is {days_late} days past due.\n\n\
         Please make your payment as soon as possible to avoid additional fees.\n\n\
         — Mason Wheeler Properties"
    );
    send_email(to, subject, &body).await
}

pub async fn send_welcome_email(to: &str, name: &str, temp_password: &str) -> Result<()> {
    let subject = "Welcome to Mason Wheeler Properties";
    let body = format!(
        "Hi {name},\n\n\
         Welcome! Your account has been created.\n\n\
         You can log in at: https://properties.mason-wheeler.com\n\n\
         Your temporary password is: {temp_password}\n\
         Please change it after your first login.\n\n\
         — Mason Wheeler Properties"
    );
    send_email(to, subject, &body).await
}
