# User creation via the admin panel

## Full flow

When an admin creates a user through the panel, the cycle is:

```
Admin fills in the form
        ↓
Account created in database (is_active = false)
Random hash injected into the password field
        ↓
Reset email sent to the user
        ↓
User clicks the link
Sets their password
        ↓
update_password → is_active = true
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
| Password setup | `is_active` is set to `true` automatically via `update_password` |
| Login | `auth_login()` blocks accounts with `is_active = false` |

---

## Builtin creation form — `UserAdminCreateForm`

The form provided by Runique (`runique::admin::UserAdminCreateForm`) exposes:

| Field | Type | Description |
| --- | --- | --- |
| `username` | Text, required | Username (min. 3 characters) |
| `email` | Email, required | Contact address + reset email recipient |
| `password` | Hidden | Random hash injected automatically — not visible in the UI |
| `is_staff` | Boolean | Read access to the admin panel |
| `is_superuser` | Boolean | Full admin access |

`is_active` is **not** in the form — accounts are always created inactive.

---

## Configuration in `admin!{}`

```rust
admin! {
    users: eihwaz_users::Model => MyForm {
        title: "Users",
        permissions: ["admin"],
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
| `reset_url` | Full reset link (absolute if `reset_password_url` is configured) |

---

## Reset URL

The constructed URL follows this pattern:

```
{base_url}/reset-password/{token}/{encrypted_email}
```

In production, configure `reset_password_url` in the builder to generate an absolute URL:

```rust
.with_admin(|a| a
    .reset_password_url("https://mysite.com/reset-password")
)
```

Without this configuration, the URL is built from the `Host` header of the HTTP request
(`http://{host}/reset-password/...`).

---

## Custom model

If the project uses its own user model (not `eihwaz_users`), it must implement
`UserEntity` and handle `is_active` in `update_password` itself.

`auth_login()` uses `BuiltinUserEntity` — projects with a custom model call
`login()` directly and control the `is_active` check in their own handler.

---

## See also

- [Macro `admin!`](/docs/en/admin/declaration/macro) — `create_form:`, `edit_form:`
- [Password reset](/docs/en/auth) — forgot/reset flow

## Back to summary

- [Admin summary](/docs/en/admin)
