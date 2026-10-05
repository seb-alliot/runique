# Session Helpers

## Import

```rust
use runique::prelude::*;
```

---

## Login

### `auth_login` — login by user_id (recommended)

Generic shortcut: automatically loads user data from the DB using only the `user_id`. Suitable for any authentication flow (registration, OAuth, magic link…).

```rust
auth_login(&session, &db, user.id).await?;
```

### `login` — full login

For cases where you already have all the data and want to control DB persistence and exclusive login.

> **Note:** If you are using your own user model (Custom Model) instead of the default table, you **must** use `login()`. The `auth_login()` wrapper systematically queries the internal `eihwaz_users` table.

```rust
login(
    &session,
    &db,
    user.id,
    &user.username,
    user.is_staff,
    user.is_superuser,
    None,    // Option<&RuniqueSessionStore> — multi-device persistence
    false,   // exclusive — invalidate other sessions
).await?;
```

### Exclusive login

To allow only one active session per user at a time, pass `exclusive: true`:

```rust
login(&session, &user, Some(&store), true).await?;
```

Or enable globally via the builder:

```rust
RuniqueApp::builder(config)
    .middleware(|m| m.with_exclusive_login(true))
```

---

## Logout

```rust
logout(&session, None).await?;

// With DB session deletion (multi-device)
logout(&session, Some(&store)).await?;
```

---

## Revoking sessions

`logout()` only ends the current session. To invalidate **all** of a user's sessions ("log out everywhere", account compromise, password change), you go through the stores.

The built-in password reset (`with_password_reset`) already does it once the new password is saved. In your own handler (password change from a profile page, compromised account…), one method is enough:

```rust
engine.close_user_sessions(user_id).await;
```

It deletes the account's sessions from the database (`eihwaz_sessions`) first, then from memory. The order matters: a session dropped from memory first could be read back from the database by a request arriving in between. A database error is traced, not returned.

> **Limit:** a store plugged in with `with_session_store()` (Redis…) can't be searched by user; its sessions are not closed.

To revoke only the **other** devices while keeping the current session, use `invalidate_other_sessions(user_id, &cookie_id)` on the DB store — this is exactly what [exclusive login](#login) does.

---

## Checks

```rust
// Is the user authenticated?
if is_authenticated(&session).await {
    // ...
}

// Get user ID from session (returns Pk = i32/i64/Uuid depending on the active feature)
if let Some(user_id) = get_user_id(&session).await {
    // ...
}

// Get username from session
if let Some(username) = get_username(&session).await {
    // ...
}
```

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
