//! E-mail client (`lettre`) — SMTP configuration, async sending, initialization from environment.
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::{env::var, sync::OnceLock};

// === Global Config ===

pub static MAILER_CONFIG: OnceLock<MailerConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct MailerConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub starttls: bool,
}

impl MailerConfig {
    pub fn from_env() -> Option<Self> {
        let host = var("SMTP_HOST").ok()?;
        let username = var("SMTP_USER").ok()?;
        let password = var("SMTP_PASS").ok()?;
        let from = var("SMTP_FROM").unwrap_or_else(|_| username.clone());
        let port = var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let starttls = var("SMTP_STARTTLS").map(|v| v == "true").unwrap_or(true);
        Some(Self {
            host,
            port,
            username,
            password,
            from,
            starttls,
        })
    }
}

pub fn mailer_init(config: MailerConfig) {
    MAILER_CONFIG.set(config).ok();
}

pub fn mailer_init_from_env() {
    if let Some(config) = MailerConfig::from_env() {
        mailer_init(config);
    }
}

pub fn mailer_configured() -> bool {
    MAILER_CONFIG.get().is_some()
}

// === Email Builder ===

pub struct Email {
    to: String,
    subject: String,
    html: Option<String>,
    text: Option<String>,
    reply_to: Option<String>,
}

impl Email {
    pub fn new() -> Self {
        Self {
            to: String::new(),
            subject: String::new(),
            html: None,
            text: None,
            reply_to: None,
        }
    }

    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to = address.into();
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.html = Some(body.into());
        self
    }

    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.text = Some(body.into());
        self
    }

    pub fn reply_to(mut self, address: impl Into<String>) -> Self {
        self.reply_to = Some(address.into());
        self
    }

    pub async fn send(self) -> Result<(), String> {
        let config = MAILER_CONFIG.get().ok_or_else(|| {
            "Mailer not configured — call mailer_init() or define SMTP_HOST/SMTP_USER/SMTP_PASS"
                .to_string()
        })?;

        let from = config
            .from
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| format!("Invalid sender address: {e}"))?;

        let to = self
            .to
            .parse::<lettre::message::Mailbox>()
            .map_err(|e| format!("Invalid recipient address: {e}"))?;

        let mut builder = Message::builder().from(from).to(to).subject(self.subject);

        if let Some(reply_to) = &self.reply_to {
            let rt = reply_to
                .parse::<lettre::message::Mailbox>()
                .map_err(|e| format!("Invalid Reply-To: {e}"))?;
            builder = builder.reply_to(rt);
        }

        let message = if let Some(html) = self.html {
            builder
                .header(ContentType::TEXT_HTML)
                .body(html)
                .map_err(|e| format!("Error constructing email: {e}"))?
        } else if let Some(text) = self.text {
            builder
                .header(ContentType::TEXT_PLAIN)
                .body(text)
                .map_err(|e| format!("Error constructing email: {e}"))?
        } else {
            return Err("Email without content".to_string());
        };

        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let transport = if config.starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
                .map_err(|e| format!("SMTP connection failed: {e}"))?
                .port(config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .map_err(|e| format!("SMTP connection failed: {e}"))?
                .port(config.port)
                .credentials(creds)
                .build()
        };

        transport
            .send(message)
            .await
            .map_err(|e| format!("Error sending email: {e}"))?;

        Ok(())
    }
}

impl Default for Email {
    fn default() -> Self {
        Self::new()
    }
}
