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
login(&session, &db, user.id, &user.username, false, false, Some(&store), true).await?;
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

`logout()` only ends the current session. To invalidate **all** of a user's sessions ("log out everywhere", account compromise, password change), call the store methods explicitly.

> **Important:** the built-in password reset (`with_password_reset`) does **not** invalidate active sessions — an already-open session (including a stolen one) stays valid until it expires. If you want to revoke sessions on password change, do it in your own handler after the update. This is a deliberate choice: the framework provides the primitives, you decide the policy.

```rust
// DB-persisted sessions (with_db_fallback)
if let Some(store) = engine
    .session_db_store
    .read()
    .ok()
    .and_then(|g| g.as_ref().cloned())
{
    store.invalidate_all(user_id).await?;
}

// In-memory sessions (default store)
if let Some(mem) = engine
    .session_store
    .read()
    .ok()
    .and_then(|g| g.as_ref().cloned())
{
    mem.invalidate_user_sessions(user_id).await;
}
```

To revoke only the **other** devices while keeping the current session, use `invalidate_other_sessions(user_id, &cookie_id)` on the DB store — this is exactly what [exclusive login](#exclusive-login) does.

---

## Checks

```rust
// Is the user authenticated?
if is_authenticated(&session).await {
    // ...
}

// Get user ID from session (returns Pk = i32 or i64)
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
