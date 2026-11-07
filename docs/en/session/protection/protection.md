# Session protection

## Automatic protection — `user_id`

Any session containing the `user_id` key is treated as authenticated and will never be removed under memory pressure (only expires normally).

This key is inserted automatically by Runique's authentication system on login.

---

## Manual protection — `session_active`

To protect a high-value anonymous session (cart, multi-step form, wizard), use `protect_session`:

```rust
use runique::prelude::*;

// Protect the session for 30 minutes
protect_session(&session, 60 * 30).await?;
```

The `session_active` key stores a future Unix timestamp. Protection expires automatically at that date — no manual cleanup needed.

To remove protection explicitly:

```rust
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
use runique::prelude::*;

pub async fn add_to_cart(request: Request) -> AppResult<Response> {
    // The item comes from the URL: /cart/add?item=42
    let mut cart: Vec<String> = request.session.get("cart").await?.unwrap_or_default();
    if let Some(item) = request.get_query("item") {
        cart.push(item.to_string());
    }
    request.session.insert("cart", &cart).await?;

    // Protect the session for 2h against emergency cleanup
    protect_session(&request.session, 60 * 60 * 2).await?;

    Ok(Redirect::to("/cart").into_response())
}

pub async fn checkout_complete(request: Request) -> AppResult<Response> {
    // Empty the cart and remove the protection
    request.session.remove::<Vec<String>>("cart").await?;
    unprotect_session(&request.session).await?;

    Ok(Redirect::to("/confirmation").into_response())
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Store & watermarks](/docs/en/session/store) | CleaningMemoryStore, purge mechanisms |
| [Usage & configuration](/docs/en/session/usage) | Access and configuration |

## Back to summary

- [Sessions](/docs/en/session)
