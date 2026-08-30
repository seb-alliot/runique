# Application, Serveur & Base de données

## Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DEBUG` | `false` | Interrupteur global dev/prod — lu **une seule fois** au démarrage via `LazyLock`. Active : niveau de log `debug`, pages d'erreur détaillées, hot reload templates admin. En production (`false`) : niveau `warn`, erreurs génériques. |
| `BASE_DIR` | `.` | Répertoire racine de l'application |
| `TZ` | `UTC` | Fuseau horaire IANA de l'application (ex : `Europe/Paris`, `America/New_York`). Accessible via `config.timezone` — à parser avec `chrono-tz` dans le projet. |
| `LANG` | locale système | Langue de la CLI (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priorité : `.env` > locale système (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP d'écoute |
| `PORT` | `3000` | Port d'écoute |
| `SECRET_KEY` | `default_secret_key` | Clé secrète (CSRF, signatures). En production (`DEBUG=false`), le **boot échoue** si elle est vide, égale au défaut, ou fait moins de 32 caractères |

---

## Base de données

### Connexion

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DATABASE_URL` | — | URL complète (prioritaire sur toutes les variables composantes) |
| `DB_ENGINE` | `sqlite` | Moteur : `postgres`, `mysql`, `mariadb`, `sqlite` |
| `DB_USER` | — | Utilisateur (requis sauf SQLite) |
| `DB_PASSWORD` | — | Mot de passe (requis sauf SQLite) |
| `DB_HOST` | `localhost` | Host |
| `DB_PORT` | `5432` / `3306` | Port (défaut selon le moteur) |
| `DB_NAME` | `local_base.sqlite` (SQLite uniquement) | Nom de la base — **requis** pour postgres/mysql/mariadb, échec au démarrage si absent |

### Pool de connexions

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_MAX_CONNECTIONS` | `100` | Taille maximale du pool |
| `DB_MIN_CONNECTIONS` | `20` | Taille minimale du pool |

### Timeouts

| Variable | Défaut | Unité | Description |
|----------|--------|-------|-------------|
| `DB_CONNECT_TIMEOUT` | `2` | secondes | Timeout d'établissement de connexion |
| `DB_ACQUIRE_TIMEOUT` | `500` | millisecondes | Timeout d'acquisition depuis le pool |
| `DB_IDLE_TIMEOUT` | `300` | secondes | Durée d'inactivité avant fermeture |
| `DB_MAX_LIFETIME` | `3600` | secondes | Durée de vie maximale d'une connexion |

### Logging SQL

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_LOGGING` | `false` | Active les logs SQL (`true`, `1`, `yes`) |

---

## Connexions secondaires — `with_custom_db`

Pour attacher une connexion additionnelle (pool Redis, PostgreSQL secondaire, client MongoDB, etc.), utilisez `.with_custom_db()` sur le builder. La valeur est stockée dans une `HashMap<TypeId, Arc<dyn Any>>` interne à `RuniqueEngine` — **pas** injectée comme `Extension` Axum. Accès dans les handlers via `engine.custom_db::<T>()` (ou son alias `engine.extension::<T>()`), qui retourne `Option<Arc<T>>`.

```rust
// main.rs
let redis = redis::Client::open("redis://127.0.0.1/")?;
let db = DatabaseConfig::from_env()?.build().connect().await?;

RuniqueAppBuilder::new(config)
    .with_database(db)
    .with_custom_db(redis)   // T: Any + Send + Sync + 'static
    .routes(url::urlpatterns())
    .build().await?
    .run().await
```

```rust
// handler
use redis::Client;

pub async fn mon_handler(mut req: Request) -> AppResult<Response> {
    let redis = req.engine.custom_db::<Client>().expect("redis non configuré");
    let mut conn = redis.get_async_connection().await?;
    // ...
}
```

Tout type implémentant `Any + Send + Sync + 'static` est accepté. Plusieurs connexions secondaires de types différents peuvent être enregistrées avec des appels `.with_custom_db()` répétés.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Assets & médias](/docs/fr/env/assets) | Fichiers statiques, médias, templates |
| [Sécurité & sessions](/docs/fr/env/securite) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
