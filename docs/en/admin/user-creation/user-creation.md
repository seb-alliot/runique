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
| Lost email | The staff resends the link from the account's page; the user asks for one through "forgot password" when the site opened that form (`with_password_reset`) |
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
- rank flags (`is_superuser`, `is_dirigeant`, `is_admin`) are not form fields. Rank is changed from the account page. `is_superuser` is set with `runique create-superuser`. Detail: [Permissions](/docs/en/admin/permission).
- on account creation, the gate injects an unknown hash and the handler sends the welcome link (`AccountLink::Welcome`)

No `create_form:` / `edit_form:` / `inject_password` flag: this is native behaviour for the accounts table.

---

## Email sending

The admin needs no declaration in `main.rs` to open the accounts it creates. It needs two things:

- **a complete mailer in `.env`** (`EMAIL_BACKEND`, or `SMTP_HOST`, `SMTP_USER` and `SMTP_PASS`, see [Mailer](/docs/en/mailer));
- **the public address** (`.with_public_url(…)`; with `RUNIQUE_ENV=dev`, it is `http://127.0.0.1:{PORT}`).

The page where the owner chooses a password is served by the admin itself, with Runique's template. `with_password_reset(…)` customizes it and opens the "forgot password" form to visitors.

**Without a complete mailer, the admin creates no account.** The staff never holds a password: an account whose link was never sent could be opened by nobody. The create page therefore goes back to the list with the message "email sending is not configured on this site", and nothing is written. Resending a password link from an account's page is refused the same way.

If the send fails while the mailer is configured (an unreachable SMTP server, for instance), the account exists: the staff resends the link from its page.

**The link is never shown to the staff**, not even in development: whoever creates the account must not be able to choose the password in its owner's place.

Templates:

| Email | Template |
| --- | --- |
| Welcome (created from the admin) | `admin/user_created_email.html` |
| Reset (button on the account's page) | `admin/reset_password_email.html` |

Tera context available in these templates:

| Variable | Value |
| --- | --- |
| `username` | Username |
| `reset_url` | Full, absolute link (on the public address) |
| `t_greeting`, `t_body`, `t_btn`… | The mail's translated texts (welcome: `t_greeting`, `t_body`, `t_btn`, `t_validity`; reset: `t_title`, `t_greeting`, `t_body`, `t_btn`, `t_ignore`) |

The email address is never put in the link: the reset page checks it against the account bound to the token.

---

## Customizing in `main.rs`

With nothing declared, the admin uses the base settings:

| Element | Base value |
| --- | --- |
| Page where the owner chooses a password | `/reset-password/{token}`, Runique's `auth/reset_password.html` template |
| Link lifetime | 1 hour |
| Rate limit of the page | 5 requests per 5 minutes and per address |
| "Forgot password" form for visitors | not served |

`with_password_reset(…)` replaces these settings and opens the form to visitors:

```rust
use std::time::Duration;

builder::new(config)
    .with_public_url("https://mysite.com")
    .with_password_reset(|mut pr| {
        // Rate limit of both pages
        pr.max_requests = 10;       // default: 5
        pr.retry_after = 600;       // default: 300 seconds
        pr.reset_route("/reset")                    // default: /reset-password
            .reset_template("auth/reset.html")      // default: Runique's template
            .forgot_route("/forgot")                // default: /forgot-password
            .forgot_template("auth/forgot.html")    // default: Runique's template
            .token_ttl(Duration::from_secs(1800))   // default: 1 hour
    })
    .with_admin(admin::config)
```

The links the admin sends follow these settings: same route, same lifetime. A template named like Runique's, placed in your `templates/` folder, replaces it with nothing to declare (`templates/auth/reset_password.html`).

The admin's two emails have their own templates, listed above. Every option of `with_password_reset` is described in [Password reset](/docs/en/auth/password-reset).

---

## Link URL

```
{public_url}/{reset_route}/{token}
```

`public_url` comes from `.with_public_url(…)`; `reset_route` is `/reset-password`, or the one given to `with_password_reset()`. In production, the app refuses to boot without a public address when the admin or the reset is enabled. In development (`RUNIQUE_ENV=dev`), the address is always `http://127.0.0.1:{PORT}`, never the request's `Host` header.

---

## See also

- [`ModelAdmin` builder](/docs/en/admin/declaration-macro) — `.form()`, `accounts_users`
- [Declaration](/docs/en/admin/declaration) — `AdminSite`
- [Password reset](/docs/en/auth/password-reset) — forgot/reset flow

## Back to summary

- [Admin summary](/docs/en/admin)
