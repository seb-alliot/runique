# Application, Serveur & Base de données

## Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `RUNIQUE_ENV` | `prod` | `dev` ou `prod`, quelle que soit la casse ; absente ou inconnue, c'est `prod` (avec un avertissement si la valeur est inconnue). Lue **une seule fois** au démarrage. En `dev` : cookie de session sans `Secure`, niveau de log `debug` (sauf `RUST_LOG`), templates relus à chaque requête, en-têtes no-cache sur `localhost`, page d'accueil par défaut si `/` n'est pas déclaré, URL publique à `http://127.0.0.1:{PORT}` (`.with_public_url()` ignorée), hôtes autorisés non validés, pas de `upgrade-insecure-requests`, pas de SRI et fichiers statiques en `no-cache` pour que le CSS modifié se recharge sans redémarrer. En `prod` : contrôles de sécurité au démarrage (`SECRET_KEY`, `with_public_url`, mailer du reset, ACME). |
| `DEBUG` | `false` | Affiche les pages d'erreur détaillées et allume tous les canaux de log (voir [Tracing](/docs/fr/configuration/tracing)) — lu **une seule fois** au démarrage. Accepte `true`/`1`/`yes`/`on` et `false`/`0`/`no`/`off`, quelle que soit la casse. Fonctionne aussi avec `RUNIQUE_ENV=prod` pour diagnostiquer un problème en direct : déconseillé, car tous les visiteurs voient alors ces pages (un avertissement s'affiche au démarrage). |
| `BASE_DIR` | `.` | Répertoire racine de l'application |
| `TZ` | `UTC` | Fuseau horaire IANA de l'application (ex : `Europe/Paris`, `America/New_York`). Accessible via `config.timezone` — à parser avec `chrono-tz` dans le projet. |
| `LANG` | locale système | Langue de la CLI (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priorité : `LANG` / `LC_ALL` / `LC_MESSAGES` du `.env` > locale système > `en`. La langue de l'application se règle dans le code (`set_lang(Lang::Fr)`) |

---

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP d'écoute |
| `PORT` | `3000` | Port d'écoute |
| `SECRET_KEY` | `default_secret_key` | Clé secrète (CSRF, signatures, chiffrement des sessions en base : la changer déconnecte leurs utilisateurs au redémarrage). `runique new` en génère une aléatoire (64 caractères hexadécimaux) dans le `.env` du projet. Pour la remplacer : n'importe quelle chaîne aléatoire d'au moins 32 caractères (`openssl rand -hex 32`). En production (`RUNIQUE_ENV=prod`), le **démarrage échoue** si elle est vide, égale au défaut, ou fait moins de 32 caractères ; en développement, un avertissement est affiché si elle est absente |

---

## Base de données

### Connexion

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DATABASE_URL` | — | URL complète. **Lue en premier** : quand elle est définie, `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT` et `DB_NAME` sont ignorés, et le moteur est déduit du schéma de l'URL (`postgres://`, `mysql://`, `sqlite://`) |
| `DB_ENGINE` | `sqlite` | Moteur, quand `DATABASE_URL` est absente : `postgres`, `mysql`, `sqlite`. Doit correspondre à la feature de `runique` compilée, sinon « driver not enabled » au démarrage |
| `DB_USER` | — | Utilisateur (requis sauf SQLite) |
| `DB_PASSWORD` | — | Mot de passe (requis sauf SQLite) |
| `DB_HOST` | `localhost` | Host |
| `DB_PORT` | `5432` / `3306` | Port (défaut selon le moteur) |
| `DB_NAME` | `local_base.sqlite` (SQLite uniquement) | Nom de la base — **requis** pour postgres/mysql, échec au démarrage si absent |

### Pool de connexions

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_MAX_CONNECTIONS` | `10` | Taille maximale du pool |
| `DB_MIN_CONNECTIONS` | `2` | Connexions gardées ouvertes au repos |

### Timeouts

| Variable | Défaut | Unité | Description |
|----------|--------|-------|-------------|
| `DB_CONNECT_TIMEOUT` | `2` | secondes | Timeout d'établissement de connexion |
| `DB_ACQUIRE_TIMEOUT` | `5000` | millisecondes | Timeout d'acquisition depuis le pool |
| `DB_IDLE_TIMEOUT` | `300` | secondes | Durée d'inactivité avant fermeture |
| `DB_MAX_LIFETIME` | `3600` | secondes | Durée de vie maximale d'une connexion |

### Régler le pool

Le pool est l'ensemble des connexions que l'application garde vers la base. Une requête HTTP qui lit ou écrit en emprunte une, le temps de sa requête SQL, puis la rend. Ces réglages concernent PostgreSQL et MySQL ; avec SQLite, Runique ouvre une seule connexion et les ignore.

- **`DB_MIN_CONNECTIONS`** : les connexions gardées ouvertes même quand personne ne visite le site. Ce n'est pas une limite. Plus la valeur est haute, moins une requête attend l'ouverture d'une connexion après une période calme ; chaque connexion ouverte occupe de la mémoire sur le serveur de base de données.
- **`DB_MAX_CONNECTIONS`** : le nombre de requêtes SQL que l'application peut exécuter en même temps. Au-delà, les requêtes attendent qu'une connexion se libère.
- **`DB_ACQUIRE_TIMEOUT`** : le temps qu'une requête attend une connexion libre. Passé ce délai, elle échoue (erreur `PoolTimedOut`, page d'erreur 500). Il joue aussi au démarrage, le temps d'ouvrir la première connexion.

Les valeurs par défaut sont volontairement petites : un projet démarre sans réglage, et un site ne peut pas occuper à lui seul le serveur de base de données. Elles conviennent à un site à trafic modéré.

**Quand les changer**

| Situation | Réglage |
|---|---|
| Des erreurs `PoolTimedOut` aux heures de pointe, des pages lentes alors que la base répond vite | augmenter `DB_MAX_CONNECTIONS` |
| Les premières requêtes après une période calme sont lentes | augmenter `DB_MIN_CONNECTIONS` |
| Vous préférez une erreur rapide à une page qui attend | baisser `DB_ACQUIRE_TIMEOUT`, sans descendre sous le temps d'ouverture d'une connexion (`PoolTimedOut` au démarrage sinon) |

**La limite du serveur de base de données**

Le serveur a son propre plafond, toutes applications confondues, réglé par `max_connections` :

| Moteur | `max_connections` par défaut | Disponible pour une application |
|---|---|---|
| PostgreSQL | généralement 100 (moins si le système ne le permet pas, décidé à l'installation) | 3 de moins : `superuser_reserved_connections` réserve 3 connexions aux superutilisateurs |
| MySQL | 151 | 151 ; une connexion supplémentaire est réservée aux comptes d'administration |

La somme des `DB_MAX_CONNECTIONS` de toutes les applications qui s'y connectent (chaque instance compte, ainsi que les outils d'administration et les sauvegardes) doit rester en dessous.

Quand plusieurs sites partagent un serveur, donnez à chacun son propre compte et limitez-le côté base, pour qu'un site ne prenne pas les connexions des autres :

```sql
-- PostgreSQL (la limite ne s'applique pas à un superutilisateur)
ALTER ROLE mon_site CONNECTION LIMIT 15;

-- MySQL
ALTER USER 'mon_site'@'localhost' WITH MAX_USER_CONNECTIONS 15;
```

### Logging SQL

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_LOGGING` | `false` | Active les logs SQL (`true`, `1`, `yes`) |

---

## Envoi de courriel

Le mailer se configure ici, et nulle part ailleurs : rien à déclarer dans le builder. Voir [Mailer](/docs/fr/mailer).

| Variable | Défaut | Description |
|----------|--------|-------------|
| `EMAIL_BACKEND` | `smtp` | `smtp` (envoi réel) ou `console` (affiche le courriel dans le terminal, pour le développement) |
| `SMTP_HOST` | — | Serveur SMTP — **requis** avec `smtp` |
| `SMTP_USER` | — | Identifiant SMTP — **requis** avec `smtp` |
| `SMTP_PASS` | — | Mot de passe SMTP — **requis** avec `smtp` |
| `SMTP_PORT` | `587` | Port du serveur |
| `SMTP_FROM` | `SMTP_USER` (`noreply@localhost` avec `console`) | Adresse d'expédition |
| `SMTP_STARTTLS` | `true` | Passe la connexion en STARTTLS |

La configuration doit être complète : avec `smtp`, s'il manque une des trois variables requises, il n'y a pas de mailer. L'application démarre quand même ; un envoi rend une erreur, et l'admin refuse de créer un compte.

---

## Connexions secondaires — `with_custom_db`

Pour attacher une connexion additionnelle (pool Redis, PostgreSQL secondaire, client MongoDB, etc.), utilisez `.with_custom_db()` sur le builder. La valeur est stockée dans une `HashMap<TypeId, Arc<dyn Any>>` interne à `RuniqueEngine` — **pas** injectée comme `Extension` Axum. Accès dans les handlers via `engine.custom_db::<T>()` (ou son alias `engine.extension::<T>()`), qui retourne `Option<Arc<T>>`.

> **`with_custom_db` ne remplace pas la base de l'ORM.** La connexion SeaORM — entités, `search!`, `save`, l'admin et les migrations — reste celle de `with_database` / `with_database_config`. Les valeurs passées à `with_custom_db` ne sont accessibles **que manuellement** via `engine.custom_db::<T>()` ; l'ORM ne les utilise jamais. Pour changer le moteur de l'ORM, voir [Base de données](/docs/fr/installation/base-de-donnees).

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
