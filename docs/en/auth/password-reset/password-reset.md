# Password Reset

[← Authentication](/docs/en/auth)

---

> The built-in reset hashes the submitted password using the password configuration on the running app. `Auto`, `Manual`, and `Custom` modes are supported. In `Delegated` mode, the built-in reset refuses to change a local password and leaves the reset token usable; use the identity provider's password recovery flow instead. See → [Password configuration](/docs/en/configuration/password).

## What the framework provides

Runique includes a complete, ready-to-use system:

- 2 auto-registered routes (forgot form + reset form)
- 2 built-in forms (`ForgotPasswordForm`, `PasswordResetForm`)
- Token generation and validation (UUID, 1h TTL, single-use)
- Automatic email sending if the mailer is configured
- Built-in rate limiting (5 requests / 5 min by default)
- Included i18n messages
- Once the reset succeeds: every session of the account closed, and the one of the browser it ran in (even signed in as another account) — in the same transaction as the password change. Showing the form signs nobody out
- Depending on the account's state: activation link (pending account), reset link (active account), "account blocked" email with no link (deactivated account) — the page always answers the same. See [Account states](/docs/en/auth/model)

---

## Activation

```rust
RuniqueApp::builder(config)
    .with_public_url("https://mysite.com")
    .with_mailer_from_env()
    .with_password_reset(|pr| pr)
    .build()
    .await?
```

The links sent by email are built on the site's public URL, set in the builder with `.with_public_url("https://mysite.com")` (see [Builder](/docs/en/configuration/builder)).

In production, the app refuses to boot without it, and without a mailer. The link is **never** built from the request's `Host` header, which an attacker chooses (`Host: evil.com` → the victim's token would go to them). In development (`RUNIQUE_ENV=dev`), the address is always `http://127.0.0.1:{PORT}`, whatever the builder was given: a test link never leads to the production site.

> Accounts are those of `eihwaz_users`, the framework's user table — extend it with `extend!{ table: "eihwaz_users", ... }`.

---

## Configuration

All options are optional — the defaults work without any changes.

```rust
.with_password_reset(|pr| pr
    .forgot_route("/forgot-password")           // default: /forgot-password
    .reset_route("/reset-password")             // default: /reset-password
    .forgot_template("auth/forgot.html")        // default: auth/forgot_password.html
    .reset_template("auth/reset.html")          // default: auth/reset_password.html
    .email_template("emails/reset.html")        // default: built-in template
    .token_ttl(Duration::from_secs(1800))       // default: 1h (std::time::Duration)
)
```

---

## Templates

Two templates can be overridden in `templates/`. When absent, the framework uses its own.

The form is rendered with `{% form.<name> %}`: it shows the fields, their errors and the CSRF token. Nothing else to add inside the `<form>`.

### `auth/forgot_password.html` — "forgot email" form

```html
{% extends "base.html" %}
{% block content %}
{% messages %}
<form method="post">
    {% form.forgot_form %}
    <button type="submit">Send the link</button>
</form>
{% endblock %}
```

Variables available in the Tera context:

| Variable | Description |
| --- | --- |
| `forgot_form` | Form with the `email` field (errors included) |
| `title` | Translated page title |
| `messages` | Flash messages: "check your inbox", or "unavailable" without a mailer |

### `auth/reset_password.html` — reset form

```html
{% extends "base.html" %}
{% block content %}
{% messages %}
{% if reset_done %}
    <p>Password updated. <a href="/">Sign in</a></p>
{% else %}
    <form method="post">
        {% form.reset_form %}
        <button type="submit">Change the password</button>
    </form>
{% endif %}
{% endblock %}
```

Available variables:

| Variable | Description |
| --- | --- |
| `reset_form` | Form with a hidden token and the `email`, `password` and `confirm` fields (errors included) |
| `token` | The token from the URL |
| `title` | Translated page title |
| `reset_done` | `true` once the password is changed: show the confirmation instead of the form |

A `<form>` without `action` posts to the page's address, `/reset-password/{token}`: the right route.

> **Automatic validations:** a password of at least 10 characters, with an uppercase letter, a lowercase letter, a digit and a special character; matching confirmation; valid and non-expired token; the typed email matching the account bound to the token.

---

## Full flow

```text
1. User clicks "Forgot password"
        ↓
2. GET /forgot-password → displays the email form
        ↓
3. POST /forgot-password → email submitted
   - If email exists: generates a token (1h), sends an email with the link
   - Account blocked by the staff: "account blocked" email, without a link
   - If email unknown: same response, in the same time (the lookup and the send
     run in the background) — does not reveal existing accounts
   - Redirects to the "forgot password" page with the "check your inbox" message
        ↓
4. User clicks the link received by email
        ↓
5. GET /reset-password/{token} → displays the reset form
   - Invalid or expired token → error message, redirect to /
        ↓
6. POST /reset-password/{token}
   - Validates the form and checks the typed email against the account bound to the token
   - Consumes the token (single-use), updates the password and activates the
     account if it was pending, closes its sessions — in one transaction
   - Renders the page again with reset_done = true (no redirect)
```

> **Active sessions after reset:** once the new password is saved, **every session of the account is closed**, on every device — a stolen one included —, and the session of the browser that did the reset. All of it happens in the password change's transaction: when the database refuses, nothing changes and the link stays usable. One link is live per account: a new request replaces the previous one. The user signs in again with the new password. Other accounts are left alone. Only Runique's stores are covered: a store plugged in with `with_session_store()` (Redis…) can't be searched by user — see [Revoking sessions](/docs/en/auth/session#revoking-sessions).

---

## Admin integration

The reset button on an account's page in the admin sends the same link, through the same code. It needs nothing more than the site's reset: `.with_password_reset(...)`, the public URL and a mailer. Without one of them, the admin shows "link unavailable": the link is **never** shown to the staff.

Admin email templates (optional, built-in otherwise):
- `templates/admin/reset_password_email.html` (reset link)
- `templates/admin/user_created_email.html` (activation link of a created account)

Variables available in these templates: `username`, `reset_url`, and the translated texts `t_greeting`, `t_body`, `t_btn`; plus `t_title` and `t_ignore` for the reset, `t_validity` for the creation.

---

## Without a configured mailer

- **In production:** the app refuses to boot with `.with_password_reset(...)` and no mailer.
- **In development:** the forgot form answers "reset unavailable" and sends nothing; the admin answers "link unavailable". The link is shown nowhere.

> To test in dev without an SMTP server: `EMAIL_BACKEND=console` prints the email to the terminal.

---

← [**Authentication**](/docs/en/auth) | [**Mailer**](/docs/en/mailer) →
