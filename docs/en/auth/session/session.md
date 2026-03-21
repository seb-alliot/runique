# Session Helpers

## Import

```rust
use runique::middleware::auth::{
    login, login_staff, logout,
    is_authenticated, get_user_id, get_username,
};
```

---

## Login

```rust
// Regular user — is_staff and is_superuser default to false
login(&session, user.id, &user.username).await?;

// Staff/admin user — with explicit rights and custom roles
login_staff(
    &session,
    user.id,
    &user.username,
    user.is_staff,
    user.is_superuser,
    user.roles(),
).await?;
```

### Exclusive login

To allow only one active session per user at a time, enable via the builder:

```rust
RuniqueApp::builder(config)
    .middleware(|m| m.with_exclusive_login(true))
```

`login` and `login_staff` will then automatically invalidate all existing sessions
for the user on each new login. No changes required in handlers.

> Disabled by default (`false`). No effect when using an external session store.

---

## Logout

```rust
logout(&session).await?;
```

---

## Checks

```rust
// Is the user authenticated?
if is_authenticated(&session).await {
    // ...
}

// Get user ID from session
if let Some(user_id) = get_user_id(&session).await {
    // ...
}

// Get username from session
if let Some(username) = get_username(&session).await {
    // ...
}
```

---

## Environment Variables

These variables control automatic redirects in the middlewares:

| Variable | Default | Description |
| --- | --- | --- |
| `REDIRECT_ANONYMOUS` | `/` | Redirect target for unauthenticated users |
| `USER_CONNECTED_URL` | `/` | Redirect target for already-authenticated users |

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
