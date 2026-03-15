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

| Variable | Défaut | Description |
|----------|--------|-------------|
| `RUNIQUE_SESSION_CLEANUP_SECS` | `60` | Intervalle du timer de cleanup (secondes) |
| `RUNIQUE_SESSION_LOW_WATERMARK` | `134217728` (128 Mo) | Seuil de cleanup proactif (octets) |
| `RUNIQUE_SESSION_HIGH_WATERMARK` | `268435456` (256 Mo) | Seuil d'urgence + refus (octets) |

```env
# Exemple : cleanup toutes les 30s, watermarks réduits pour serveur limité
RUNIQUE_SESSION_CLEANUP_SECS=30
RUNIQUE_SESSION_LOW_WATERMARK=67108864
RUNIQUE_SESSION_HIGH_WATERMARK=134217728
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
    // Durée de vie des sessions
    .with_session_duration(time::Duration::hours(2))
    // Watermarks personnalisés
    .with_session_memory_limit(64 * 1024 * 1024, 128 * 1024 * 1024)
    // Intervalle de cleanup
    .with_session_cleanup_interval(30)
    .build()
    .await?;
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Store & watermarks](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/store/store.md) | CleaningMemoryStore, mécanismes de purge |
| [Protection](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/protection/protection.md) | Protection des sessions |

## Retour au sommaire

- [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md)
