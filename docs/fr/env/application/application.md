# Application, Serveur & Base de données

## Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DEBUG` | `false` | Interrupteur global dev/prod — lu **une seule fois** au démarrage. Accepte `true`/`1`/`yes`/`on` et `false`/`0`/`no`/`off`, quelle que soit la casse. Active : niveau de log `debug` (sauf `RUST_LOG`), pages d'erreur détaillées, templates relus à chaque requête, page d'accueil par défaut si `/` n'est pas déclaré. En production (`false`) : niveau `warn`, erreurs génériques, contrôles de sécurité au démarrage (`SECRET_KEY`, `with_public_url`, ACME). |
| `BASE_DIR` | `.` | Répertoire racine de l'application |
| `TZ` | `UTC` | Fuseau horaire IANA de l'application (ex : `Europe/Paris`, `America/New_York`). Accessible via `config.timezone` — à parser avec `chrono-tz` dans le projet. |
| `LANG` | locale système | Langue de la CLI (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priorité : `LANG` / `LC_ALL` / `LC_MESSAGES` du `.env` > locale système > `en`. La langue de l'application se règle dans le code (`set_lang(Lang::Fr)`) |

---

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP d'écoute |
| `PORT` | `3000` | Port d'écoute |
| `SECRET_KEY` | `default_secret_key` | Clé secrète (CSRF, signatures, chiffrement des sessions en base : la changer déconnecte leurs utilisateurs au redémarrage). `runique new` en génère une aléatoire (64 caractères hexadécimaux) dans le `.env` du projet. Pour la remplacer : n'importe quelle chaîne aléatoire d'au moins 32 caractères (`openssl rand -hex 32`). En production (`DEBUG=false`), le **démarrage échoue** si elle est vide, égale au défaut, ou fait moins de 32 caractères ; en debug, un avertissement est affiché |

---

## Base de données

### Connexion

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DATABASE_URL` | — | URL complète. **Lue en premier** : quand elle est définie, `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT` et `DB_NAME` sont ignorés, et le moteur est déduit du schéma de l'URL (`postgres://`, `mysql://`, `mariadb://`, `sqlite://`) |
| `DB_ENGINE` | `sqlite` | Moteur, quand `DATABASE_URL` est absente : `postgres`, `mysql`, `mariadb`, `sqlite`. Doit correspondre à la feature de `runique` compilée, sinon « driver not enabled » au démarrage |
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

RuniqueApp::builder(config)
    .with_database(db)
    .with_custom_db(redis)   // T: Any + Send + Sync + 'static
    .routes(url::routes())
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
| [Sécurité & sessions](/docs/fr/env/securite) | Middlewares, CSP, Sessions |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
