# Session Helpers

## Import

```rust
use runique::prelude::*;
```

---

## Login

### `login` — open the session

Two steps: `authenticate_user` checks the credentials (password, active and activated account), then `login` records the account in the session. `login` renews the session id and the CSRF token, and writes only the id: the account is read from the database on every request (`request.user`).

```rust
match authenticate_user(&db, &username, &password).await {
    Some(user) => login(&session, &user, None, false).await?,
    None => { /* invalid credentials: generic message */ }
}
```

Parameters: the account (`&impl RuniqueUser`), multi-device persistence (`Option<&RuniqueSessionStore>`; the default store already saves signed-in sessions to the database) and exclusive login.

`login` checks by itself that the account may sign in (`can_sign_in()`: active and activated), whatever path loaded it. Otherwise nothing is written and it returns `LoginError::CannotSignIn`:

```rust
match login(&session, &user, None, false).await {
    Ok(()) => { /* signed in */ }
    Err(LoginError::CannotSignIn) => { /* inactive, or not activated yet */ }
    Err(LoginError::Session(e)) => { /* the session store failed */ }
}
```

> **An account loaded another way** (after a registration, OAuth, a magic link) signs in with the same `login`. It must come from the server — an account just created, an identity verified — never from an id received in the request.

### Activate, then sign in

`BuiltinUserEntity::activate_account` activates a pending account (`is_active` + `activated_at`) and returns the updated account, ready for `login`. An account activated before, or deactivated since by the staff, is left untouched (`None`): reactivation stays the staff's decision.

```rust
if let Some(user) = BuiltinUserEntity::activate_account(&db, id).await? {
    login(&session, &user, None, false).await?;
}
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

`logout()` clears the **whole** session (flash messages included) and gives it a new id: a flash message meant for the next page is added **after** the call.

When the deletion fails (database down), `logout()` returns the error and the session stays **whole**, cookie included: the user can try again. Don't show "signed out" then. Likewise, `login` as another account first closes the previous session; when it can't, the new sign-in is refused (`LoginError::Session`).

---

## Revoking sessions

`logout()` only ends the current session. To invalidate **all** of a user's sessions ("log out everywhere", account compromise, password change), you go through the stores.

The built-in password reset (`with_password_reset`) already does it once the new password is saved. In your own handler (password change from a profile page, compromised account…), one method is enough:

```rust
request.engine.close_user_sessions(user_id).await?;
```

It deletes the account's sessions from the database (`eihwaz_sessions`), then their copies in memory. The database is the authority: every request of a signed-in session checks its row again, so a session deleted from the database is refused everywhere, another instance included. When the database fails, nothing else is touched and the error is returned: the sessions are still alive, don't present the operation as done.

> **Custom store:** a store plugged in with `with_session_store()` (Redis…) can't be searched by user. Its sessions close anyway: `close_user_sessions` and a completed reset increase `session_version` on the account, and a session opened under an older version ends at its next request.

To revoke only the **other** devices while keeping the current session, use `invalidate_other_sessions(user_id, &cookie_id)` on the DB store — this is exactly what [exclusive login](#login) does.

---

## Checks

```rust
// Is the user authenticated?
if is_authenticated(&session).await {
    // ...
}

// The signed-in account, read from the database on every request (like Django's `request.user`)
if let Some(user) = &request.user {
    let user_id = user.id; // Pk = i32/i64/Uuid depending on the active feature
    let username = &user.username;
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
