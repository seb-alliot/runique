# Mailer — sending emails

[← Back](/docs/en)

---

## Configuration

The mailer is initialized in the application builder. Two modes are available:

### Via environment variables (recommended)

```rust
RuniqueAppBuilder::new(config)
    .with_mailer_from_env()
    // ...
```

`.env` variables:

```env
# Backend: "smtp" (default) or "console" (dev — prints to terminal)
EMAIL_BACKEND=smtp

SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASS=secret
SMTP_FROM=noreply@example.com   # optional, defaults to SMTP_USER
SMTP_STARTTLS=true              # optional, defaults to true
```

### Manually

```rust
use runique::prelude::MailerConfig;

RuniqueAppBuilder::new(config)
    .with_mailer(MailerConfig {
        backend: Default::default(),  // Smtp
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        password: "secret".to_string(),
        from: "noreply@example.com".to_string(),
        starttls: true,
    })
```

---

## Backends

| Backend | Behaviour |
| --- | --- |
| `smtp` | Real delivery via SMTP (production) |
| `console` | Prints the email to the terminal (development) |

In development, `EMAIL_BACKEND=console` avoids needing a local SMTP server. The full email (from, to, subject, body) is printed to the logs.

---

## Sending an email

### Shorthand — plain text

```rust
use runique::prelude::dispatch_email;

dispatch_email("user@example.com", "Welcome", "Your account has been created.").await?;
```

### Full builder — HTML, templates, reply-to

```rust
use runique::prelude::Email;

Email::new()
    .to("user@example.com")
    .subject("Welcome to the platform")
    .html("<h1>Hello!</h1><p>Your account is active.</p>")
    .reply_to("support@example.com")
    .send()
    .await?;
```

### With a Tera template

```rust
use runique::prelude::Email;
use runique::context;

let ctx = context! { "username" => "Alice", "url" => "https://example.com/confirm/abc" };

Email::new()
    .to("alice@example.com")
    .subject("Confirm your address")
    .template(&tera, "emails/confirmation.html", ctx.into())?
    .send()
    .await?;
```

The template is a standard Tera file in your `templates/` directory:

```html
{# templates/emails/confirmation.html #}
<h1>Hello {{ username }}!</h1>
<p><a href="{{ url }}">Confirm your email address</a></p>
```

---

## `Email` builder methods

| Method | Description |
| --- | --- |
| `.to(address)` | Recipient |
| `.subject(text)` | Subject line |
| `.html(body)` | HTML body |
| `.text(body)` | Plain text body |
| `.reply_to(address)` | Reply-To header |
| `.template(&tera, name, ctx)` | Renders a Tera template as the HTML body. Returns `Result`. |
| `.send().await` | Sends the email. Returns `Result<(), String>`. |

> **Priority:** if both `.html()` and `.template()` are called, the last one wins. `.text()` is used only if no HTML body is set.

---

## Usage in a handler

```rust
use runique::prelude::dispatch_email;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();

    if form.is_valid().await {
        let email = form.cleaned_string("email").unwrap_or_default();
        let username = form.cleaned_string("username").unwrap_or_default();

        // ... save to DB ...

        if let Err(e) = dispatch_email(
            &email,
            "Welcome!",
            &format!("Hello {username}, your account is now active."),
        )
        .await
        {
            tracing::warn!("Email error: {e}");
        }

        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => { "form" => &form });
    request.render("register.html")
}
```

> **Send errors:** `.send()` and `dispatch_email` return `Result<(), String>`. In production, log the error and continue — a failed email should not crash the request.

---

← [**Back**](/docs/en)
