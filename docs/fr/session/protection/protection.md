# Protection des sessions

## Protection automatique — `user_id`

Toute session contenant la clé `user_id` est considérée comme authentifiée et ne sera jamais supprimée sous pression mémoire (uniquement expirée normalement).

Cette clé est insérée automatiquement par le système d'authentification de Runique à la connexion.

---

## Protection manuelle — `session_active`

Pour protéger une session anonyme à valeur (panier, formulaire multi-étapes, wizard), utilisez `protect_session` :

```rust
use runique::prelude::*;

// Protège la session pendant 30 minutes
protect_session(&session, 60 * 30).await?;
```

La clé `session_active` stocke un timestamp Unix futur. La protection expire automatiquement à cette date — aucune clé à nettoyer manuellement.

Pour retirer la protection explicitement :

```rust
unprotect_session(&session).await?;
```

### Logique de protection

```
is_protected(record) = true si :
  - record contient "user_id"
  - OU record contient "session_active" avec un timestamp futur
```

---

## Cas d'usage — protection d'un panier

```rust
use runique::prelude::*;

pub async fn add_to_cart(request: Request) -> AppResult<Response> {
    // L'article vient de l'URL : /cart/add?item=42
    let mut cart: Vec<String> = request.session.get("cart").await?.unwrap_or_default();
    if let Some(item) = request.get_query("item") {
        cart.push(item.to_string());
    }
    request.session.insert("cart", &cart).await?;

    // Protéger la session 2h contre le cleanup d'urgence
    protect_session(&request.session, 60 * 60 * 2).await?;

    Ok(Redirect::to("/cart").into_response())
}

pub async fn checkout_complete(request: Request) -> AppResult<Response> {
    // Vider le panier et retirer la protection
    request.session.remove::<Vec<String>>("cart").await?;
    unprotect_session(&request.session).await?;

    Ok(Redirect::to("/confirmation").into_response())
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Store & watermarks](/docs/fr/session/store) | CleaningMemoryStore, mécanismes de purge |
| [Usage & configuration](/docs/fr/session/usage) | Accès et configuration |

## Retour au sommaire

- [Sessions](/docs/fr/session)
