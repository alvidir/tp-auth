use std::error::Error;
use std::env;
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre_email::EmailBuilder;

use crate::constants;

fn get_mailer() -> Result<SmtpTransport, Box<dyn Error>> {
    let smtp_username = env::var(constants::ENV_SMTP_USERNAME)?;
    let smtp_password = env::var(constants::ENV_SMTP_PASSWORD)?;
    let smtp_transport = env::var(constants::ENV_SMTP_TRANSPORT)?;

    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpClient::new_simple(&smtp_transport)?
        .credentials(creds)
        .transport();

    Ok(mailer)
}


pub fn send_email(to: &str, subject: &str, body: &str) -> Result<(), Box<dyn Error>> {
    let from = env::var(constants::ENV_SMTP_ORIGIN)?;
    let email = EmailBuilder::new()
        .to(to)
        .from(from)
        .subject(subject)
        .html(body)
        .build()?;

    get_mailer()?.send(email.into())?;
    Ok(())
}