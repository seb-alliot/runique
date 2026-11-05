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
- rank flags (`is_superuser`, `is_dirigeant`, `is_admin`) are not form fields. Rank is changed from the account page. `is_superuser` is set with `runique create-superuser`. Detail: [Permissions](/docs/en/admin/permission).
- on account creation, the gate injects an unknown hash and the handler sends the welcome link (`AccountLink::Welcome`)

No `create_form:` / `edit_form:` / `inject_password` flag: this is native behaviour for the accounts table.

---

## Email sending

The welcome link is sent only when all three conditions hold:

- a mailer is configured (`SMTP_*` variables in `.env`, see [Mailer](/docs/en/mailer));
- password reset is enabled (`.with_password_reset(…)` in the builder): the link leads to its page;
- the public address is known (`.with_public_url(…)`; with `RUNIQUE_ENV=dev`, it is `http://127.0.0.1:{PORT}`).

Otherwise the account is still created, and the staff sees a "link unavailable" warning. **The link is never shown to the staff**, not even in development: whoever creates the account must not be able to choose the password in its owner's place. The user can always ask for a link later through "forgot password".

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

## Link URL

```
{public_url}/{reset_route}/{token}
```

`public_url` comes from `.with_public_url(…)`; `reset_route` is the one of `with_password_reset()` (`/reset-password` by default). In production, the app refuses to boot without a public address when the admin or the reset is enabled. In debug only, the address is built from the request's `Host` header.

---

## See also

- [`ModelAdmin` builder](/docs/en/admin/declaration-macro) — `.form()`, `accounts_users`
- [Declaration](/docs/en/admin/declaration) — `AdminSite`
- [Password reset](/docs/en/auth/password-reset) — forgot/reset flow

## Back to summary

- [Admin summary](/docs/en/admin)
