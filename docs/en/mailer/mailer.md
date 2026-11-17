# Mailer — sending emails

[← Back](/docs/en)

---

## Configuration

The mailer is configured in `.env`, and nowhere else: there is nothing to declare in the application's builder. Runique reads these variables at the first send.

```env
# Backend: "smtp" (default) or "console" (dev — prints to the terminal)
EMAIL_BACKEND=smtp

SMTP_HOST=smtp.example.com
SMTP_PORT=587                   # optional, default = 587
SMTP_USER=user@example.com
SMTP_PASS=secret
SMTP_FROM=noreply@example.com   # optional, default = SMTP_USER
SMTP_STARTTLS=true              # optional, default = true
```

**The configuration must be complete.** With the `smtp` backend, `SMTP_HOST`, `SMTP_USER` and `SMTP_PASS` are all three required: if one is missing, the mailer counts as absent, exactly like an empty `.env`. The `console` backend needs nothing else.

An absent mailer does not stop the application from starting: a site that sends no email has nothing to configure. What depends on it says so when it needs it:

| Who sends | Without a complete mailer |
| --- | --- |
| Your code (`Email::new()…send()`, `dispatch_email`) | `send()` returns an error |
| The admin (creating an account, resending a password link) | the action is refused before anything is written, with a message |
| The "forgot password" page declared by `with_password_reset` | in production, the application refuses to start |

`mailer_configured()` tells whether the mailer is complete, so a handler can decide what to do before sending.

---

## Backends

| Backend | Behaviour |
| --- | --- |
| `smtp` | Real delivery via SMTP (production) |
| `console` | Prints the email to the terminal (development) |

In development, `EMAIL_BACKEND=console` avoids needing a local SMTP server. The full email (from, to, subject, body) is printed to the logs.

---

## The admin and the mailer

The admin sends emails itself: it never holds a password, so each account it creates gets a link to choose its own.

**Base behaviour.** With a complete mailer in `.env`, nothing else has to be written. The admin sends the link, and serves the page where the owner chooses a password itself, with Runique's template.

**Without a complete mailer, the admin refuses.** Before anything is written, it checks that the mailer is fully configured:

| Action in the admin | Without a complete mailer |
| --- | --- |
| Creating an account | refused: no account is created, the page goes back to the list |
| Resending an account's password link | refused: no link is created |

In both cases the admin shows "Email sending is not configured on this site: this action cannot be done." An account whose link was never sent could be opened by nobody: better not to create it.

**Customizing.** The page's templates, the route, the link's lifetime and the rate limit are set in `main.rs`, with `with_password_reset(…)`. See [User creation](/docs/en/admin/user-creation).

---

## Sending an email

### Shorthand — plain text

```rust
use runique::prelude::*;

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
use runique::prelude::*;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form("register");
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "register_form" => &form });
            return request.render("register.html");
        }
    };

    let email = form.cleaned::<String>("email").unwrap_or_default();
    let username = form.cleaned::<String>("username").unwrap_or_default();

    // ... save to DB ...

    // A failed send does not undo the sign-up: it is only logged.
    if let Err(e) = dispatch_email(
        &email,
        "Welcome!",
        &format!("Hello {username}, your account is active."),
    )
    .await
    {
        tracing::warn!("Email error: {e}");
    }

    Ok(Redirect::to("/").into_response())
}
```

> **Send errors:** `.send()` and `dispatch_email` return `Result<(), String>`. In production, log the error and continue — a failed email should not crash the request.

---

← [**Back**](/docs/en)
