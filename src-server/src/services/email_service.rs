// Author: Quadri Atharu

use anyhow::{anyhow, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Arc;

#[derive(Clone)]
pub struct EmailService {
    transport: Option<Arc<AsyncSmtpTransport<Tokio1Executor>>>,
    from_email: String,
    enabled: bool,
}

impl EmailService {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        smtp_username: &str,
        smtp_password: &str,
        from_email: &str,
        enabled: bool,
    ) -> Self {
        let transport = if enabled {
            let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
            let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)
                .map_err(|e| tracing::warn!("SMTP starttls failed: {}", e))
                .ok()
                .map(|builder| builder.port(smtp_port).credentials(creds).build());
            transport.map(Arc::new)
        } else {
            None
        };

        Self {
            transport,
            from_email: from_email.to_string(),
            enabled,
        }
    }

    pub fn from_env() -> Self {
        let enabled = std::env::var("EMAIL_ENABLED")
            .unwrap_or_default()
            .parse::<bool>()
            .unwrap_or(false);

        Self::new(
            &std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.example.com".to_string()),
            std::env::var("SMTP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(587),
            &std::env::var("SMTP_USERNAME").unwrap_or_default(),
            &std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            &std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@haqly-erp.com".to_string()),
            enabled,
        )
    }

    pub async fn send_password_reset_email(
        &self,
        to: &str,
        token: &str,
        reset_url: &str,
    ) -> Result<()> {
        let full_url = format!("{}?token={}", reset_url, token);

        let html_body = format!(
            "<html><body style=\"font-family:Arial,sans-serif;color:#333;\">\
            <div style=\"max-width:600px;margin:0 auto;padding:20px;\">\
            <h2 style=\"color:#1a5276;\">HAQLY ERP - Password Reset</h2>\
            <p>You requested a password reset for your HAQLY ERP account.</p>\
            <p>Click the link below to reset your password. This link expires in 1 hour.</p>\
            <p><a href=\"{}\" style=\"background-color:#1a5276;color:#fff;padding:10px 20px;text-decoration:none;border-radius:4px;\">Reset Password</a></p>\
            <p style=\"color:#999;font-size:12px;\">If you did not request this, you can safely ignore this email.</p>\
            </div></body></html>",
            full_url
        );

        let text_body = format!(
            "HAQLY ERP - Password Reset\n\nYou requested a password reset.\n\nReset your password here: {}\n\nThis link expires in 1 hour.\n\nIf you did not request this, you can safely ignore this email.",
            full_url
        );

        self.send_email(to, "HAQLY ERP - Password Reset", &html_body, &text_body)
            .await
    }

    pub async fn send_welcome_email(&self, to: &str, username: &str) -> Result<()> {
        let html_body = format!(
            "<html><body style=\"font-family:Arial,sans-serif;color:#333;\">\
            <div style=\"max-width:600px;margin:0 auto;padding:20px;\">\
            <h2 style=\"color:#1a5276;\">Welcome to HAQLY ERP</h2>\
            <p>Hello {},</p>\
            <p>Your HAQLY ERP account has been created successfully.</p>\
            <p>You can now log in and start managing your business with Nigerian accounting compliance built in.</p>\
            <p style=\"color:#999;font-size:12px;\">If you have any questions, please contact your system administrator.</p>\
            </div></body></html>",
            username
        );

        let text_body = format!(
            "Welcome to HAQLY ERP\n\nHello {},\n\nYour account has been created successfully. You can now log in and start using HAQLY ERP.",
            username
        );

        self.send_email(to, "Welcome to HAQLY ERP", &html_body, &text_body)
            .await
    }

    pub async fn send_invoice_notification(
        &self,
        to: &str,
        invoice_number: &str,
    ) -> Result<()> {
        let html_body = format!(
            "<html><body style=\"font-family:Arial,sans-serif;color:#333;\">\
            <div style=\"max-width:600px;margin:0 auto;padding:20px;\">\
            <h2 style=\"color:#1a5276;\">HAQLY ERP - Invoice Notification</h2>\
            <p>Invoice <strong>{}</strong> has been generated and is ready for review.</p>\
            <p>Please log in to your HAQLY ERP dashboard to view the details.</p>\
            </div></body></html>",
            invoice_number
        );

        let text_body = format!(
            "HAQLY ERP - Invoice Notification\n\nInvoice {} has been generated and is ready for review.\n\nPlease log in to your dashboard to view the details.",
            invoice_number
        );

        let subject = format!("Invoice {} - HAQLY ERP", invoice_number);
        self.send_email(to, &subject, &html_body, &text_body).await
    }

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<()> {
        if !self.enabled {
            tracing::info!(
                "Email disabled - would send to: {}, subject: {}, body: {}",
                to,
                subject,
                text_body
            );
            return Ok(());
        }

        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| anyhow!("SMTP transport not configured"))?;

        let email = Message::builder()
            .from(self.from_email.parse().map_err(|e| anyhow!("Invalid from email: {}", e))?)
            .to(to.parse().map_err(|e| anyhow!("Invalid to email: {}", e))?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| anyhow!("Failed to build email: {}", e))?;

        transport
            .send(email)
            .await
            .map_err(|e| anyhow!("Failed to send email: {}", e))?;

        tracing::info!("Email sent successfully to: {}", to);
        Ok(())
    }
}
