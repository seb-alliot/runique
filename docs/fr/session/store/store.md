# Store & watermarks

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

Les sessions protégées ne sont jamais sacrifiées en passe 1.

---

---

## Persistance — fallback base de données

Par défaut, le store est purement en mémoire. Lorsque la feature `orm` est active, un fallback base de données est **activé automatiquement** pour les sessions authentifiées — aucune configuration supplémentaire n'est nécessaire.

- **Lecture** : mémoire d'abord, DB en cas de miss (session survit à un redémarrage)
- **Écriture** : synchrone en mémoire + asynchrone en DB pour les sessions avec `user_id`
- **Sessions anonymes** : jamais persistées en DB

> La table `eihwaz_sessions` doit exister — elle est créée par les migrations du framework (`runique migration up`). Sans elle, la première requête qui cherche une session en base échoue (`no such table`).
>
> Avec un store personnalisé (`with_session_store(...)`), ce fallback n'est pas installé, et Runique ne sait pas révoquer les sessions d'un utilisateur (déconnexion des autres appareils, réinitialisation de mot de passe) : c'est au store de le faire.

---

## Plusieurs instances

Le store mémoire appartient à un seul processus. Derrière un répartiteur de charge avec plusieurs instances de l'application :

- **Les sessions anonymes** (jeton CSRF, messages flash, tout ce qui est stocké avant la connexion) ne vivent que dans la mémoire de l'instance qui les a créées. Une requête envoyée à une autre instance ne les retrouve pas : un formulaire échoue au contrôle CSRF, un message flash disparaît.
- **Les sessions authentifiées** sont partagées par la base : la base fait autorité. À chaque requête, une instance qui a la session en mémoire vérifie encore que sa ligne existe en base. Une déconnexion, une réinitialisation de mot de passe ou un `close_user_sessions` sur une instance supprime la ligne : les autres instances refusent leur copie dès la requête suivante.

Tant qu'il n'existe pas de store partagé pour les sessions anonymes, envoyez chaque visiteur toujours vers la même instance (sessions « collantes », ou affinité de session, sur le répartiteur) si vos pages utilisent des formulaires avant la connexion. Avec une seule instance, rien de tout cela ne s'applique.

---

## Configuration

Tous les seuils décrits ci-dessus sont réglables depuis le builder, dans le bloc `.middleware()`. Aucune construction manuelle du store n'est nécessaire — le framework instancie `CleaningMemoryStore` et applique ces valeurs.

```rust
use runique::prelude::*;
use runique::tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    // TTL d'inactivité avant expiration (sessions authentifiées)
    .with_session_duration(Duration::hours(2))
    .middleware(|m| {
        m
            // Low / high watermark — en octets
            .with_session_memory_limit(64 * 1024 * 1024, 128 * 1024 * 1024)
            // Intervalle du timer de purge périodique — en secondes
            .with_session_cleanup_interval(30)
            // Un seul appareil connecté à la fois par utilisateur
            .with_exclusive_login(true)
    })
    .build()
    .await?;
```

| Mécanisme (section ci-dessus) | Méthode builder | Défaut |
|-------------------------------|-----------------|--------|
| Timer périodique | `with_session_cleanup_interval(secs)` | 60 s |
| Low / High watermark | `with_session_memory_limit(low, high)` | 128 Mo / 256 Mo |
| TTL session authentifiée | `with_session_duration(Duration)` | 24 h |
| TTL session anonyme | `with_anonymous_session_duration(Duration)` | 5 min |
| Login exclusif (un appareil) | `with_exclusive_login(bool)` | `false` |

> Les watermarks s'expriment en **octets** : `64 * 1024 * 1024` = 64 Mo. La durée du timer est en **secondes**. `with_session_duration` s'appelle sur le builder ; les quatre autres méthodes, dans `.middleware(|m| …)`. Les deux durées sont des `runique::tower_sessions::cookie::time::Duration`.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Protection](/docs/fr/session/protection) | Protection des sessions |
| [Usage & configuration](/docs/fr/session/usage) | Accès et configuration |

## Retour au sommaire

- [Sessions](/docs/fr/session)
