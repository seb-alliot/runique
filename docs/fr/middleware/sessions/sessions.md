# Sessions (middleware)

## Store par défaut

Runique utilise `MemoryStore` par défaut (données en mémoire, perdues au redémarrage).

---

## Durées de session

| Durée | Usage |
|-------|-------|
| `Duration::minutes(30)` | Sessions courtes (sécurité) |
| `Duration::hours(2)` | Usage standard |
| `Duration::hours(24)` | Défaut Runique |
| `Duration::days(7)` | "Se souvenir de moi" |

---

## Configuration

```rust
// Durée de session personnalisée
let app = RuniqueApp::builder(config)
    .with_session_duration(time::Duration::hours(2))
    .build()
    .await?;
```

### Store personnalisé (production)

```rust
// Exemple avec un store Redis
let app = RuniqueApp::builder(config)
    .middleware(|m| m.with_session_store(RedisStore::new(client)))
    .build()
    .await?;
```

---

## Accès à la session dans les handlers

```rust
pub async fn dashboard(request: Request) -> AppResult<Response> {
    // Lire une valeur de session
    let user_id: Option<i32> = request.session
        .get("user_id")
        .await
        .ok()
        .flatten();

    // Écrire une valeur
    let _ = request.session.insert("last_visit", "2026-02-06").await;
}
```

> Pour le système de sessions complet avec watermarks, voir [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md) | Protection CSRF |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
