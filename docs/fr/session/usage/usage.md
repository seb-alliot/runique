# Accès à la session & configuration

## Accès à la session dans les handlers

```rust
pub async fn handler(request: Request) -> AppResult<Response> {
    // Lire
    let user_id: Option<i32> = request.session.get("user_id").await.ok().flatten();

    // Écrire
    request.session.insert("cart_id", 42).await?;

    // Supprimer une clé
    request.session.remove::<i32>("cart_id").await?;

    // Détruire la session entière
    request.session.flush().await?;
}
```

---

## Configuration `.env`

```rust,ignore
.middleware(|m| {
    m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
     .with_session_cleanup_interval(5)
})
```

---

## Sécurité des cookies par défaut

Les cookies de session sont configurés avec les attributs de sécurité suivants par défaut :

| Attribut | Valeur | Description |
| --- | --- | --- |
| `HttpOnly` | `true` | Toujours activé — inaccessible au JavaScript |
| `SameSite` | `Strict` | Bloque les requêtes cross-site |
| `Secure` | `true` en production | HTTPS uniquement (désactivé en debug) |

Ces valeurs sont appliquées automatiquement par le builder et ne peuvent pas être modifiées sans toucher au framework.

---

## Configuration via le builder

```rust
let app = RuniqueApp::builder(config)
    // Durée de vie des sessions (disponible directement sur le builder)
    .with_session_duration(time::Duration::hours(2))
    // Watermarks et intervalle de cleanup (via .middleware())
    .middleware(|m| {
        m.with_session_memory_limit(64 * 1024 * 1024, 128 * 1024 * 1024)
         .with_session_cleanup_interval(30)
    })
    .build()
    .await?;
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Store & watermarks](/docs/fr/session/store) | CleaningMemoryStore, mécanismes de purge |
| [Protection](/docs/fr/session/protection) | Protection des sessions |

## Retour au sommaire

- [Sessions](/docs/fr/session)
