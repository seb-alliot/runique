# User creation via the admin panel

## Full flow

When an admin creates a user through the panel, the cycle is:

```
Admin fills in the form
        ↓
Account created in database, pending (is_active = false, activated_at empty)
Random hash injected into the password field
        ↓
Reset email sent to the user
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
| `password` field | Random hash injected — the user never sees this value |
| Email | Reset link sent to the address provided in the form |
| Password setup | The pending account is activated: `is_active = true` and `activated_at` set. An account activated before (deactivated since) is **not** reactivated |
| Login | Refused until the account is active and activated (`can_sign_in()`) |
| Lost email | The user asks for a new link through "forgot password" |
| Activation by an admin | Not possible: ticking `is_active` on a pending account is refused — the first activation belongs to the email's owner |

---

## Builtin creation form — `UserAdminCreateForm`

The form provided by Runique (`runique::admin::UserAdminCreateForm`) exposes:

| Field | Type | Description |
| --- | --- | --- |
| `username` | Text, required | Username (min. 3 characters) |
| `email` | Email, required | Contact address + reset email recipient |
| `password` | Hidden | Random hash injected automatically — not visible in the UI |
| `is_staff` | Boolean | Read access to the admin panel |

`is_active` and `is_superuser` are **not** in the form — accounts are always created inactive, and superuser status can never be granted from the admin panel itself (protection against self-privilege-escalation).

---

## Configuration in `admin!{}`

```rust
admin! {
    users: runique_users::Model => MyForm {
        title: "Users",
        create_form: runique::admin::UserAdminCreateForm,
        edit_form: crate::forms::UserEditForm,
    }
}
```

The `create_form:` field tells the daemon to use `UserAdminCreateForm` on creation
and to enable the `inject_password` flag on the resource.

---

## Email sending

The email is sent if the mailer is configured (`SMTP_*` variables in `.env`).

Without a mailer (development), the reset link is displayed in a flash message.

The default email template is `admin/user_created_email.html`. It can be overridden
via `AdminConfig::reset_password_email_template("my_template/email.html")`.

Tera context available in the template:

| Variable | Value |
| --- | --- |
| `username` | Username |
| `email` | Email address |
| `reset_url` | Full reset link, absolute (on `SITE_URL`) |

---

## Reset URL

The constructed URL follows this pattern:

```
{SITE_URL}{reset_route}/{token}/{encrypted_email}
```

`SITE_URL` comes from the `.env` (or `.site_url(…)` in the builder); `reset_route` is the one of `with_password_reset()` (`/reset-password` by default). In production, the app refuses to boot without `SITE_URL`. In debug only, the URL is built from the `Host` header of the HTTP request
(`http://{host}/reset-password/...`).

---

## See also

- [Macro `admin!`](/docs/en/admin/declaration-macro) — `create_form:`, `edit_form:`
- [Password reset](/docs/en/auth/password-reset) — forgot/reset flow

## Back to summary

- [Admin summary](/docs/en/admin)
