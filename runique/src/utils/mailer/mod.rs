//! E-mail client (`lettre`) — SMTP + console backends, async sending, Tera template support.
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::{env::var, sync::OnceLock};

// ─── Backend ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub enum MailerBackend {
    #[default]
    Smtp,
    Console,
}

// ─── Config ──────────────────────────────────────────────────────────────────

pub static MAILER_CONFIG: OnceLock<MailerConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct MailerConfig {
    pub backend: MailerBackend,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub starttls: bool,
}

impl MailerConfig {
    pub fn from_env() -> Option<Self> {
        let backend = match var("EMAIL_BACKEND").as_deref().unwrap_or("smtp") {
            "console" => MailerBackend::Console,
            _ => MailerBackend::Smtp,
        };

        match backend {
            MailerBackend::Console => Some(Self {
                backend: MailerBackend::Console,
                host: String::new(),
                port: 0,
                username: String::new(),
                password: String::new(),
                from: var("SMTP_FROM").unwrap_or_else(|_| "noreply@localhost".to_string()),
                starttls: false,
            }),
            MailerBackend::Smtp => {
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
                    backend: MailerBackend::Smtp,
                    host,
                    port,
                    username,
                    password,
                    from,
                    starttls,
                })
            }
        }
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

// ─── Email builder ────────────────────────────────────────────────────────────

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

    /// Renders a Tera template with a `tera::Context` and sets it as the HTML body.
    ///
    /// Build the context with the `context!` macro:
    /// ```rust,no_run
    /// # use runique::prelude::Email;
    /// # use runique::context;
    /// # async fn example(tera: &tera::Tera, user_email: &str, username: &str, confirm_url: &str) -> Result<(), String> {
    /// let ctx = context! { "username" => username, "url" => confirm_url };
    /// Email::new().to(user_email).subject("Bienvenue").template(tera, "emails/welcome.html", ctx)?.send().await?;
    /// # Ok(()) }
    /// ```
    pub fn template(
        mut self,
        tera: &tera::Tera,
        template_name: &str,
        ctx: tera::Context,
    ) -> Result<Self, String> {
        let rendered = tera
            .render(template_name, &ctx)
            .map_err(|e| format!("Template error ({template_name}): {e}"))?;
        self.html = Some(rendered);
        Ok(self)
    }

    pub async fn send(self) -> Result<(), String> {
        let config = MAILER_CONFIG.get().ok_or_else(|| {
            "Mailer not configured — call .with_mailer_from_env() in the builder or set EMAIL_BACKEND/SMTP_* in .env"
                .to_string()
        })?;

        if self.html.is_none() && self.text.is_none() {
            return Err("Email without content".to_string());
        }

        match config.backend {
            MailerBackend::Console => self.send_console(config),
            MailerBackend::Smtp => self.send_smtp(config).await,
        }
    }

    fn send_console(self, config: &MailerConfig) -> Result<(), String> {
        let body = self
            .html
            .as_deref()
            .or(self.text.as_deref())
            .unwrap_or("(no content)");

        println!(
            "\n{}\n  From:    {}\n  To:      {}\n  Subject: {}{}\n\n{}\n{}",
            "─".repeat(60),
            config.from,
            self.to,
            self.subject,
            self.reply_to
                .as_deref()
                .map(|r| format!("\n  Reply-To: {r}"))
                .unwrap_or_default(),
            body,
            "─".repeat(60),
        );
        Ok(())
    }

    async fn send_smtp(self, config: &MailerConfig) -> Result<(), String> {
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

// ─── Shorthand ────────────────────────────────────────────────────────────────

/// Sends a plain-text email in one call.
/// For HTML, templates, or reply-to, use `Email::new()` instead.
pub async fn dispatch_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
    Email::new().to(to).subject(subject).text(body).send().await
}
