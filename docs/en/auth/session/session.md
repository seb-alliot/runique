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
|-----------------------|--------|------------------------------------------------------|
| `REDIRECT_ANONYMOUS` | `/` | Redirect target for unauthenticated users |
| `USER_CONNECTED_URL` | `/` | Redirect target for already-authenticated users |

---

## See also

| Section | Description |
| --- | --- |
| [User model](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/model/model.md) | Built-in model, `RuniqueUser` trait |
| [Middlewares & CurrentUser](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/middleware/middleware.md) | Route protection |

## Back to summary

- [Authentication](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md)
