//! E-mail client (`lettre`) — SMTP + console backends, async sending, Tera template support.
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::{env::var, sync::OnceLock};

// ─── Backend ─────────────────────────────────────────────────────────────────

/// Which transport `Email::send()` uses to deliver a message.
#[derive(Debug, Clone, Default)]
pub enum MailerBackend {
    /// Sends over SMTP, using the credentials/host in [`MailerConfig`]. Default.
    #[default]
    Smtp,
    /// Prints the email to stdout instead of sending it — for local dev.
    Console,
}

// ─── Config ──────────────────────────────────────────────────────────────────

/// Global mailer configuration, set once via [`mailer_init`] or
/// [`mailer_init_from_env`] and read by every [`Email::send`] call.
pub static MAILER_CONFIG: OnceLock<MailerConfig> = OnceLock::new();

/// SMTP/console mailer configuration. Build it with [`MailerConfig::from_env`]
/// or construct it directly, then register it with [`mailer_init`].
#[derive(Clone)]
pub struct MailerConfig {
    /// Transport used to deliver emails.
    pub backend: MailerBackend,
    /// SMTP server host (unused for the `Console` backend).
    pub host: String,
    /// SMTP server port (unused for the `Console` backend).
    pub port: u16,
    /// SMTP authentication username (unused for the `Console` backend).
    pub username: String,
    /// SMTP authentication password (unused for the `Console` backend). Never
    /// printed — `Debug` redacts it as `***` to avoid leaking it via logs.
    pub password: String,
    /// `From:` address used on every sent email.
    pub from: String,
    /// Whether to upgrade the SMTP connection with STARTTLS.
    pub starttls: bool,
}

// Debug manuel : ne jamais imprimer le mot de passe SMTP en clair (fuite via logs).
impl std::fmt::Debug for MailerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailerConfig")
            .field("backend", &self.backend)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"***")
            .field("from", &self.from)
            .field("starttls", &self.starttls)
            .finish()
    }
}

impl MailerConfig {
    /// Builds a config from environment variables: `EMAIL_BACKEND` (`"console"`
    /// or `"smtp"`, defaults to `"smtp"`), `SMTP_FROM`, and for the SMTP backend
    /// `SMTP_HOST`/`SMTP_USER`/`SMTP_PASS` (required — returns `None` if any is
    /// missing), plus optional `SMTP_PORT` (default `587`) and `SMTP_STARTTLS`
    /// (default `true`).
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

/// Registers the mailer configuration globally. A no-op if it was already set
/// (e.g. called twice) — the first configuration wins.
pub fn mailer_init(config: MailerConfig) {
    MAILER_CONFIG.set(config).ok();
}

/// Builds a [`MailerConfig`] from environment variables (see
/// [`MailerConfig::from_env`]) and registers it via [`mailer_init`]. Leaves the
/// mailer unconfigured if the required SMTP variables are missing.
pub fn mailer_init_from_env() {
    if let Some(config) = MailerConfig::from_env() {
        mailer_init(config);
    }
}

/// Whether [`mailer_init`] (or [`mailer_init_from_env`]) has been called.
pub fn mailer_configured() -> bool {
    MAILER_CONFIG.get().is_some()
}

// ─── Email builder ────────────────────────────────────────────────────────────

/// Builder for a single outgoing email — chain the setters, then call
/// [`Email::send`]. Requires either [`Email::html`], [`Email::text`], or
/// [`Email::template`] to be called, and the global mailer to be configured
/// (see [`mailer_init`]/[`mailer_init_from_env`]).
pub struct Email {
    to: String,
    subject: String,
    html: Option<String>,
    text: Option<String>,
    reply_to: Option<String>,
}

impl Email {
    /// Starts a new, empty email builder.
    pub fn new() -> Self {
        Self {
            to: String::new(),
            subject: String::new(),
            html: None,
            text: None,
            reply_to: None,
        }
    }

    /// Sets the recipient address.
    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to = address.into();
        self
    }

    /// Sets the email subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    /// Sets a raw HTML body directly. For a Tera-rendered body, use
    /// [`Email::template`] instead.
    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.html = Some(body.into());
        self
    }

    /// Sets a plain-text body. Ignored if [`Email::html`]/[`Email::template`]
    /// was also called — `send()` prefers HTML when both are set.
    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.text = Some(body.into());
        self
    }

    /// Sets the `Reply-To` address.
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
    /// # use runique::macros::helper::ContextHelper;
    /// # async fn example(tera: &tera::Tera, user_email: &str, username: &str, confirm_url: &str) -> Result<(), String> {
    /// let ctx = context! { "username" => username, "url" => confirm_url };
    /// Email::new().to(user_email).subject("Bienvenue").template(tera, "emails/welcome.html", ctx.into())?.send().await?;
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

    /// Sends the email through the globally configured backend. Fails if the
    /// mailer isn't configured, if neither an HTML nor a text body was set, or
    /// if the addresses/transport are invalid.
    pub async fn send(self) -> Result<(), String> {
        let config = MAILER_CONFIG.get().ok_or_else(|| {
            if let Some(level) = crate::utils::runique_log::get_log()
                .mailer
                .as_ref()
                .and_then(|m| m.send)
            {
                crate::runique_log!(level, to = %self.to, subject = %self.subject, "send — mailer not configured");
            }
            "Mailer not configured — call .with_mailer_from_env() in the builder or set EMAIL_BACKEND/SMTP_* in .env"
                .to_string()
        })?;

        if self.html.is_none() && self.text.is_none() {
            return Err("Email without content".to_string());
        }

        let backend = match config.backend {
            MailerBackend::Console => "console",
            MailerBackend::Smtp => "smtp",
        };
        if let Some(level) = crate::utils::runique_log::get_log()
            .mailer
            .as_ref()
            .and_then(|m| m.send)
        {
            crate::runique_log!(level, to = %self.to, subject = %self.subject, backend, "send");
        }
        let result = match config.backend {
            MailerBackend::Console => self.send_console(config),
            MailerBackend::Smtp => self.send_smtp(config).await,
        };
        if let Err(ref e) = result
            && let Some(level) = crate::utils::runique_log::get_log()
                .mailer
                .as_ref()
                .and_then(|m| m.send)
        {
            crate::runique_log!(level, error = %e, backend, "send error");
        }
        result
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
