use anyhow::{Context, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use zeroize::Zeroize;

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

    let mut pw = password.to_string();
    let creds = Credentials::new(account.email.clone(), pw.clone());
    pw.zeroize();

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

    let result = mailer.send(email).await.context("Failed to send email");
    drop(mailer);
    result.map(|_| ())
}
