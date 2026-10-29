# User creation via the admin panel

## Full flow

When an admin creates a user through the panel, the cycle is:

```
Admin fills in the form
        ↓
Account created in database, pending (is_active = false, activated_at empty)
Random hash injected into the password field
        ↓
Welcome / reset email sent to the user
        ↓
User clicks the link
Sets their password
        ↓
Activation: is_active = true, activated_at set
        ↓
User can log in
```

---

## What the framework does automatically

| Step | Builtin behaviour |
| --- | --- |
| Creation via admin | `is_active = false` — account is inactive at creation |
| `password` field | Random hash injected by the gate — staff never sees it |
| Email | "Choose your password" link sent to the form address |
| Password setup | The pending account is activated: `is_active = true` and `activated_at` set. An account activated before (deactivated since) is **not** reactivated |
| Login | Refused until the account is active and activated (`can_sign_in()`) |
| Lost email | The user asks for a new link through "forgot password" |
| Activation by an admin | Not possible: ticking `is_active` on a pending account is refused — the first activation belongs to the email's owner |

---

## Accounts form — no separate `create_form:`

With the `ModelAdmin` builder, create/edit forms for accounts come from the model (`eihwaz_users` extended via `extend!`). You configure the visible fields:

```rust
fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .title("Users")
        .list(|l| {
            l.display([Username, Email, IsActive])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email]))
}

// in AdminSite:
site.accounts_users(accounts())
```

Key points:

- `password` is a **secret** column: refused in `display`, `search`, `filter`, `fields`, etc.
- rank flags (`is_superuser`, `is_dirigeant`, `is_admin`) never go through the form — the gate owns them
- on account creation, the gate injects an unknown hash and the handler sends the welcome link (`AccountLink::Welcome`)

No `create_form:` / `edit_form:` / `inject_password` flag: this is native behaviour for the accounts table.

---

## Email sending

The email is sent if the mailer is configured (`SMTP_*` variables in `.env`).

Without a mailer (development), the reset link is displayed in a flash message.

The default welcome email template is `admin/user_created_email.html`.
The reset template (from the account page) is `admin/reset_password_email.html`.

Tera context available in these templates:

| Variable | Value |
| --- | --- |
| `username` | Username |
| `email` | Email address |
| `reset_url` | Full reset link, absolute (on the public URL) |

---

## Reset URL

The constructed URL follows this pattern:

```
{public_url}{reset_route}/{token}/{encrypted_email}
```

`public_url` comes from `.with_public_url(…)` in the builder; `reset_route` is the one of `with_password_reset()` (`/reset-password` by default). In production, the app refuses to boot without it. In debug only, the URL is built from the `Host` header of the HTTP request
(`http://{host}/reset-password/...`).

---

## See also

- [`ModelAdmin` builder](/docs/en/admin/declaration-macro) — `.form()`, `accounts_users`
- [Declaration](/docs/en/admin/declaration) — `AdminSite`
- [Password reset](/docs/en/auth/password-reset) — forgot/reset flow

## Back to summary

- [Admin summary](/docs/en/admin)
