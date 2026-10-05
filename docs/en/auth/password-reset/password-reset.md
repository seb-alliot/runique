# Password Reset

[← Authentication](/docs/en/auth)

---

> **`Auto` mode required for the built-in reset:** the integrated route (`with_password_reset`) reads the form value after `finalize()` and writes it to the database. In `Auto` mode, `finalize()` hashes the password automatically — everything works correctly. In `Manual`, `Custom`, or `Delegated` mode, `finalize()` does not hash: the password would be stored in plaintext. If you are not using `PasswordConfig::auto()`, write your own reset route that hashes the received value. See → [Password configuration](/docs/en/configuration/password)

## What the framework provides

Runique includes a complete, ready-to-use system:

- 2 auto-registered routes (forgot form + reset form)
- 2 built-in forms (`ForgotPasswordForm`, `PasswordResetForm`)
- Token generation and validation (UUID, 1h TTL, single-use)
- Automatic email sending if the mailer is configured
- Built-in rate limiting (5 requests / 5 min by default)
- Included i18n messages
- Automatic logout when the reset form is accessed
- Depending on the account's state: activation link (pending account), reset link (active account), "account blocked" email with no link (deactivated account) — the page always answers the same. See [Account states](/docs/en/auth/model)

---

## Activation

```rust
RuniqueAppBuilder::new(config)
    .with_mailer_from_env()
    .with_password_reset(|pr| pr)
    .build()
    .await?
```

The links sent by email are built on `SITE_URL`, the site's public URL, read from the `.env` (or `.site_url("https://mysite.com")` in the builder):

```env
SITE_URL=https://mysite.com
```

In production, the app refuses to boot without `SITE_URL`: otherwise the link would be built from the request's `Host` header, which an attacker chooses (`Host: evil.com` → the victim's token goes to them). In debug, the `Host` stays the fallback, with a warning.

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
    .success_redirect("/login")                 // default: /
)
```

---

## Templates

Two templates can be created in `templates/`. If absent, the framework uses its own default templates.

### `auth/forgot_password.html` — "forgot email" form

```html
{% extends "base.html" %}
{% block content %}
<form method="post">
    {{ form.html | safe }}
    <button type="submit">Send reset link</button>
</form>

{% if form.errors.email %}
    <p class="error">{{ form.errors.email }}</p>
{% endif %}
{% endblock %}
```

Available Tera context variables:

| Variable | Description |
| --- | --- |
| `form` | Form with `email` field |
| `form.errors` | Validation errors |

### `auth/reset_password.html` — reset form

```html
{% extends "base.html" %}
{% block content %}
<form method="post">
    {{ form.html | safe }}
    <button type="submit">Change password</button>
</form>

{% if form.errors.password %}
    <p class="error">{{ form.errors.password }}</p>
{% endif %}
{% endblock %}
```

Available variables:

| Variable | Description |
| --- | --- |
| `form` | Form with `password` + `confirm` fields (token and email are hidden) |
| `form.errors` | Validation errors |

> **Automatic validations:** minimum 10-character password, matching confirmation, valid and non-expired token.

---

## Full flow

```text
1. User clicks "Forgot password"
        ↓
2. GET /forgot-password → displays the email form
        ↓
3. POST /forgot-password → email submitted
   - If email exists: generates a token (1h), sends an email with the link
   - If email unknown: same response (security — does not reveal existing accounts)
        ↓
4. User clicks the link received by email
        ↓
5. GET /reset-password/{token}/{encrypted_email} → displays the reset form
   - Invalid or expired token → error
        ↓
6. POST /reset-password/{token}/{encrypted_email}
   - Validates the form (password min 10 chars, confirmation)
   - Consumes the token (single-use)
   - Updates password in DB
   - Redirects to success_redirect (default: /)
```

> **Active sessions after reset:** once the new password is saved, **every session of the account is closed**, on every device — a stolen one included. The user signs in again with the new password. Other accounts are left alone. Only Runique's stores are covered: a store plugged in with `with_session_store()` (Redis…) can't be searched by user — see [Revoking sessions](/docs/en/auth/session#revoking-sessions).

---

## Admin integration

If the admin is activated with `.user_resource()`, the panel can send reset links directly from the user detail view.

The link is built on `SITE_URL` and the configured reset route (`reset_route`, `/reset-password` by default): nothing to set on the admin side. Without a mailer, the link is shown in a flash message.

Optional admin email templates (framework defaults otherwise):
- `templates/admin/reset_password_email.html`
- `templates/admin/user_created_email.html`

Available variables in these templates: `username`, `email`, `reset_url`.

---

## Without a configured mailer

The mailer is optional. If absent:
- In development: the reset link appears in a flash message in the admin
- On the forgot form: the success message is shown but no email is sent

> To test in dev without an SMTP server: `EMAIL_BACKEND=console` prints the email to the terminal.

---

← [**Authentication**](/docs/en/auth) | [**Mailer**](/docs/en/mailer) →
