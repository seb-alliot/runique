# Session protection

## Automatic protection — `user_id`

Any session containing the `user_id` key is treated as authenticated and will never be removed under memory pressure (only expires normally).

This key is inserted automatically by Runique's authentication system on login.

---

## Manual protection — `session_active`

To protect a high-value anonymous session (cart, multi-step form, wizard), use `protect_session`:

```rust
use runique::middleware::auth::protect_session;

// Protect the session for 30 minutes
protect_session(&session, 60 * 30).await?;
```

The `session_active` key stores a future Unix timestamp. Protection expires automatically at that date — no manual cleanup needed.

To remove protection explicitly:

```rust
use runique::middleware::auth::unprotect_session;

unprotect_session(&session).await?;
```

### Protection logic

```
is_protected(record) = true if:
  - record contains "user_id"
  - OR record contains "session_active" with a future timestamp
```

---

## Use case — protecting a shopping cart

```rust
pub async fn add_to_cart(request: Request, item: Item) -> AppResult<Response> {
    // Add item to cart
    request.session.insert("cart", &cart).await?;

    // Protect the session for 2 hours against emergency cleanup
    protect_session(&request.session, 60 * 60 * 2).await?;

    Ok(redirect("/cart"))
}

pub async fn checkout_complete(request: Request) -> AppResult<Response> {
    // Clear cart and remove protection
    request.session.remove::<Cart>("cart").await?;
    unprotect_session(&request.session).await?;

    Ok(redirect("/confirmation"))
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Store & watermarks](https://github.com/seb-alliot/runique/blob/main/docs/en/session/store/store.md) | CleaningMemoryStore, purge mechanisms |
| [Usage & configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/session/usage/usage.md) | Access and configuration |

## Back to summary

- [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)
