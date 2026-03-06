# Sessions

## Vue d'ensemble

Runique utilise `CleaningMemoryStore` comme store de session par défaut — un wrapper autour d'un `HashMap` en mémoire qui ajoute :

- Purge automatique des sessions expirées (timer configurable)
- Protection par watermarks contre les fuites mémoire et les attaques par inondation de sessions
- Protection des sessions à valeur (utilisateurs authentifiés, paniers, formulaires multi-étapes)

Les données sont perdues au redémarrage du serveur. Pour la persistance, utilisez un store externe (Redis, base de données).

---

## CleaningMemoryStore

### Pourquoi

Le `MemoryStore` de tower-sessions n'implémente pas de nettoyage des sessions expirées. Sans purge, chaque requête d'un bot sans cookies crée une session qui ne sera jamais supprimée — la mémoire croît indéfiniment.

`CleaningMemoryStore` résout ce problème avec trois mécanismes :

| Mécanisme | Déclencheur | Comportement |
|-----------|-------------|--------------|
| Timer périodique | Toutes les 60s (configurable) | Supprime toutes les sessions expirées |
| Low watermark | 128 Mo (configurable) | Purge asynchrone des sessions anonymes expirées |
| High watermark | 256 Mo (configurable) | Purge synchrone d'urgence + refus (503) si insuffisant |

### Estimation de la taille

Chaque record est estimé à : `24 octets (UUID + expiry) + taille JSON des données`.

Une alerte est loggée si un record dépasse 50 Ko (image ou fichier stocké en session par erreur).

---

## Système de watermarks

### Low watermark (128 Mo par défaut)

Lorsque la taille totale du store dépasse ce seuil, un cleanup non-bloquant est lancé en arrière-plan via `tokio::spawn`. Il supprime les sessions **anonymes expirées** sans interrompre la requête en cours.

### High watermark (256 Mo par défaut)

Lorsque la taille dépasse ce seuil au moment de créer une session :

1. **Passe 1** — supprime les sessions anonymes expirées
2. **Passe 2** — si toujours dépassé, supprime toutes les sessions expirées (y compris authentifiées)
3. **Refus** — si toujours dépassé, retourne `503 Service Unavailable`

Les sessions protégées (voir ci-dessous) ne sont jamais sacrifiées en passe 1.

---

## Protection des sessions

### Automatique — `user_id`

Toute session contenant la clé `user_id` est considérée comme authentifiée et ne sera jamais supprimée sous pression mémoire (uniquement expirée normalement).

Cette clé est insérée automatiquement par le système d'authentification de Runique à la connexion.

### Manuelle — `session_active`

Pour protéger une session anonyme à valeur (panier, formulaire multi-étapes, wizard), utilisez `protect_session` :

```rust
use runique::middleware::auth::protect_session;

// Protège la session pendant 30 minutes
protect_session(&session, 60 * 30).await?;
```

La clé `session_active` stocke un timestamp Unix futur. La protection expire automatiquement à cette date — aucune clé à nettoyer manuellement.

Pour retirer la protection explicitement :

```rust
use runique::middleware::auth::unprotect_session;

unprotect_session(&session).await?;
```

### Logique de protection

```
is_protected(record) = true si :
  - record contient "user_id"
  - OU record contient "session_active" avec un timestamp futur
```

---

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

## Cas d'usage — protection d'un panier

```rust
pub async fn add_to_cart(request: Request, item: Item) -> AppResult<Response> {
    // Ajouter l'article au panier
    request.session.insert("cart", &cart).await?;

    // Protéger la session 2h contre le cleanup d'urgence
    protect_session(&request.session, 60 * 60 * 2).await?;

    Ok(redirect("/cart"))
}

pub async fn checkout_complete(request: Request) -> AppResult<Response> {
    // Vider le panier et retirer la protection
    request.session.remove::<Cart>("cart").await?;
    unprotect_session(&request.session).await?;

    Ok(redirect("/confirmation"))
}
```

---

## Prochaines étapes

← [**Authentification**](https://github.com/seb-alliot/runique/blob/main/docs/fr/13-authentification.md) | [**Variables d'environnement**](https://github.com/seb-alliot/runique/blob/main/docs/fr/15-env.md) →
