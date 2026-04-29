use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::Account;

pub async fn send_email(
    account: &Account,
    password: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    let email = Message::builder()
        .from(account.email.parse().context("Invalid from address")?)
        .to(to.parse().context("Invalid to address")?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .context("Failed to build email")?;

    let creds = Credentials::new(account.email.clone(), password.to_string());

    let mailer = if account.smtp_port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&account.smtp_host)?
            .port(account.smtp_port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&account.smtp_host)?
            .port(account.smtp_port)
            .credentials(creds)
            .build()
    };

    mailer.send(email).await.context("Failed to send email")?;
    Ok(())
}
