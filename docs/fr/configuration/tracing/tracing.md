# Tracing structuré

Runique expose un système de tracing opt-in par domaine via `RuniqueLog`. Chaque domaine s'active indépendamment — rien n'est loggué par défaut.

## Activation rapide en développement

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.dev())   // tout à DEBUG si DEBUG=true
    // ...
```

`.dev()` est un no-op si `DEBUG` n'est pas `true` — utilisable inconditionnellement.

---

## Configuration granulaire

```rust
.with_log(|l| l
    .forms(|f| f
        .validate(Level::DEBUG)
        .finalize(Level::DEBUG)
    )
    .admin(|a| a
        .crud(Level::INFO)
        .auth(Level::WARN)
    )
    .auth(|a| a
        .login(Level::INFO)
        .reset(Level::WARN)
    )
    .mailer(|m| m.send(Level::INFO))
    .builder(|b| b
        .templates(Level::INFO)
        .middleware(Level::DEBUG)
        .routes(Level::INFO)
        .statics(Level::INFO)
    )
    .rate_limit(Level::WARN)
)
```

---

## Domaines disponibles

### `forms` — Pipeline formulaire

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `field` | Enregistrement d'un champ | nom, type, required |
| `set_value` | Valeur assignée par `fill()` | nom, valeur (password masqué) |
| `validate` | Résultat de validation | champ, ok/error, nb global errors |
| `render` | Rendu HTML | champ, ok/error |
| `finalize` | Hash/move fichier | champ, ok/error |

### `admin` — Panel admin

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `auth` | Vérification accès + CSRF fail | resource, action |
| `crud` | Dispatch + résultat create/edit/delete | resource, action, ok/error |
| `list` | Dispatch + résultat liste | resource, rows, total, page |
| `bulk` | Actions de masse | resource, action |

### `auth` — Authentification

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `login` | Création de session | user_id, username, is_superuser, exclusive, db_persist |
| `reset` | Flux reset mot de passe | email, étape (token généré / email envoyé / invalide / ok / error) |

### `mailer`

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `send` | Envoi email | backend, to, subject, ok/error |

### `builder` — Démarrage (one-time)

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `templates` | Chargement Tera | internal, user, total |
| `registry` | Ressources admin | count |
| `middleware` | Stack middleware | count + slot + name pour chaque entrée |
| `routes` | Registry URL | count |
| `statics` | Fichiers statiques | static_url, static_dir, media_url, media_dir |

### Champs plats sur `RuniqueLog`

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `rate_limit` | Requête bloquée | ip, retry_after |
| `csrf` | Token CSRF détecté dans une URL GET | path |
| `session` | Opérations session store | event |
| `db` | Requêtes base de données | query, duration |
| `host_validation` | Hôte rejeté | host |

---

## Erreurs inconditionnelles (toujours actives)

Indépendamment de la config tracing, certaines erreurs sont toujours loggées via `tracing::error!` :

- **Template invalide** — si un template Tera échoue au chargement, le nom du template et l'erreur (avec numéro de ligne) sont loggués avant l'arrêt du démarrage.

---

## Voir aussi

- [Variables d'environnement](/docs/fr/configuration/variables) — `RUST_LOG`, `DEBUG`
- [Configuration programmatique](/docs/fr/configuration/builder) — `.with_log()`
