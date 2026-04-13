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
