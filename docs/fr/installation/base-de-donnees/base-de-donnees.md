# Base de données

## Choisir le moteur (features Cargo)

| Feature | Ce qu'elle apporte |
|---|---|
| `orm` (par défaut) | SeaORM : entités, requêtes, migrations, sessions. **Aucun pilote** : le code compile et `makemigrations` fonctionne, mais l'application ne peut se connecter à aucune base. |
| `sqlite`, `postgres`, `mysql` | Le pilote du moteur. Chacun active aussi `orm`. |
| `all-databases` | Les trois pilotes à la fois, pour les outils multi-moteurs (CLI, CI). |

Une application active **un seul** moteur : en activer deux est une erreur de compilation (sauf via `all-databases`). `orm` n'a pas besoin d'être écrite.

MariaDB n'est pas pris en charge : SeaORM ne connaît que ces trois moteurs, et refuse une adresse `mariadb://`.

---

## Versions de SeaORM

Runique impose une version **exacte** de SeaORM. Les `Cargo.toml` générés par `runique new` la reprennent automatiquement.

| Runique | SeaORM / sea-orm-migration |
|---|---|
| 4.1.0-preview | `=2.0.4` |
| 4.0.0 | `=2.0.4` |

Ne changez pas cette version à la main : quand vous mettez Runique à jour, alignez `Cargo.toml` et `migration/Cargo.toml` sur la version de la nouvelle release. Une version différente dans le projet est refusée par Cargo à la résolution des dépendances.

---

## Changer de moteur : trois endroits à modifier ensemble

Un projet créé par `runique new` est en SQLite. Pour passer à un autre moteur, modifiez **les trois** :

**1. `Cargo.toml`** — la feature de `runique` :

```toml
[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
```

**2. `migration/Cargo.toml`** — la feature de `runique` **et** le pilote de `sea-orm-migration` :

```toml
[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
sea-orm-migration = { version = "=2.0.4", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
] }
```

| Moteur | Feature `runique` | Feature `sea-orm-migration` |
|---|---|---|
| SQLite | `sqlite` | `sqlx-sqlite` |
| PostgreSQL | `postgres` | `sqlx-postgres` |
| MySQL | `mysql` | `sqlx-mysql` |

**3. `.env`** — `DB_ENGINE` et `DATABASE_URL` :

```env
DB_ENGINE=postgres
DATABASE_URL=postgres://runique_user:secure_password@localhost:5432/mon_projet
```

Si `.env` désigne un moteur qui n'est pas compilé, l'application refuse de démarrer avec « driver not enabled » ; le message indique le moteur compilé, pour aligner l'un ou l'autre.

Puis appliquez les migrations sur la nouvelle base : `runique migration up`.

### Exemples de `DATABASE_URL`

```env
DATABASE_URL="sqlite://data.db?mode=rwc"                        # fichier créé s'il n'existe pas
DATABASE_URL=postgres://user:password@localhost:5432/ma_base
DATABASE_URL=mysql://user:password@localhost:3306/ma_base
```

Un chemin SQLite relatif est lu depuis le dossier où l'application est lancée : lancez `cargo run` et `runique migration` depuis la racine du projet.

---

## Comment Runique choisit la connexion

La même règle sert à l'application, à `runique makemigrations` et à `runique migration up | down | status`. Deux sources sont lues dans le `.env` :

| Source | Contenu |
| --- | --- |
| `DATABASE_URL` | l'adresse complète, par exemple `postgres://app:secret@db.local:5432/shop` |
| clés séparées | `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME` |

Runique n'impose ni l'une ni l'autre. `DATABASE_URL` est lue en premier ; les clés prennent le relais quand elle est absente ou incomplète, et restent en réserve quand elle est complète.

### Le tableau

| `DATABASE_URL` | Clés séparées | Ce que fait Runique |
| --- | --- | --- |
| complète | peu importe | utilise l'adresse. Si la connexion échoue, voir « Quand l'adresse ne répond pas » |
| incomplète | suffisantes | prévient en console, se connecte par les clés, puis rappelle que `DATABASE_URL` est incomplète |
| incomplète | partielles | refuse de démarrer, en disant pourquoi l'adresse est incomplète et ce qui manque aux clés |
| incomplète, vers un serveur ou sans moteur reconnu | aucune | refuse de démarrer : SQLite n'est jamais pris à la place |
| `sqlite://` sans nom de fichier | aucune | prévient, puis utilise le fichier `local_base.sqlite` |
| absente | suffisantes | construit l'adresse avec les clés |
| absente | partielles | refuse de démarrer, en disant ce qui manque |
| absente | aucune | SQLite, fichier `local_base.sqlite` |

Une clé présente mais sans valeur (`DB_USER=`) compte comme absente, `DATABASE_URL` comprise. Une ligne commentée n'est jamais lue : c'est la façon normale de désactiver une source.

### Quand une adresse est incomplète

Dans quatre cas seulement :

| Cas | Exemple |
| --- | --- |
| son début ne désigne aucun moteur pris en charge | `mariadb://…`, `app:secret@db.local/shop` |
| elle est illisible | un port qui n'est pas un nombre (`:543a`), un mot de passe contenant `/`, `#` ou `?` non encodé |
| elle vise un serveur sans nommer de base | `postgres://app:secret@db.local`, `postgres://app:secret@db.local/` |
| elle demande SQLite sans nommer de fichier | `sqlite://`, `sqlite:` |

Tout le reste est tenté tel quel, et c'est le serveur qui tranche. Sur PostgreSQL, `?dbname=shop` compte comme un nom de base.

### Ce que Runique laisse au pilote

Pour une adresse à laquelle il manque un morceau qui n'est pas la base, Runique reprend le comportement de SeaORM et de son pilote, sans rien ajouter :

| Morceau absent | PostgreSQL | MySQL |
| --- | --- | --- |
| utilisateur | `PGUSER` de la machine, sinon le nom du compte système | `root` |
| mot de passe | `PGPASSWORD` de la machine, sinon aucun | aucun |
| hôte | `PGHOST` de la machine, sinon `localhost` | `localhost` |
| port | `PGPORT` de la machine, sinon 5432 | 3306 |

Une adresse sans mot de passe (`postgres://app@db.local/shop`) ou sans utilisateur (`postgres://db.local/shop`) est donc tentée : certains serveurs l'acceptent. Pour PostgreSQL, les morceaux absents sont complétés par les variables `PG*` de la machine, pas par les clés `DB_*` du `.env`.

Autres écritures acceptées : `sqlite:data.db`, `sqlite:///chemin/absolu.db`, `sqlite::memory:` ; un hôte en IPv6 (`postgres://[::1]/shop`) ; une connexion par socket (`?host=/var/run/postgresql`). Un paramètre inconnu après le `?` est laissé au pilote.

### Les clés séparées

| Clé | Serveur (PostgreSQL, MySQL) | SQLite |
| --- | --- | --- |
| `DB_ENGINE` | obligatoire : `postgres` (ou `postgresql`), `mysql` | `sqlite` |
| `DB_USER` | obligatoire | ignorée |
| `DB_PASSWORD` | obligatoire | ignorée |
| `DB_HOST` | facultative, `localhost` par défaut | ignorée |
| `DB_PORT` | facultative, celui du moteur par défaut | ignorée |
| `DB_NAME` | obligatoire : le nom de la base | facultative : le fichier, `local_base.sqlite` par défaut |

- La casse et les espaces de `DB_ENGINE` ne comptent pas. Une autre valeur est une erreur qui liste les moteurs acceptés.
- **Des clés sans `DB_ENGINE` sont une erreur** : sans le moteur, aucune adresse ne peut être construite, et Runique ne prend pas SQLite à sa place.
- **L'utilisateur et le mot de passe s'écrivent tels quels** : Runique les encode en construisant l'adresse. Un mot de passe `p@ss/w:rd#1` se donne ainsi, sans rien encoder. Dans `DATABASE_URL`, c'est à vous de l'encoder (`p%40ss%2Fw%3Ard%231`), sinon l'adresse est illisible.
- Une valeur qui contient `#` ou `$` se met entre apostrophes dans le `.env` (`DB_PASSWORD='pa$$#1'`) : sans elles, le lecteur du fichier coupe la valeur au `#` et remplace `$nom` par une autre variable.

### Quand l'adresse ne répond pas

Une `DATABASE_URL` complète dont la connexion échoue (serveur arrêté, mot de passe refusé, base inexistante) ne bloque pas tout de suite : Runique prévient en console et **essaie les clés séparées**.

| Clés séparées | Suite |
| --- | --- |
| suffisantes | connexion par les clés ; un échec bloque le démarrage |
| partielles | démarrage bloqué : le message donne l'échec de l'adresse et ce qui manque aux clés |
| aucune | démarrage bloqué sur l'échec de l'adresse |

Le second essai a lieu même si les clés désignent la même base : il montre si l'échec venait de `DATABASE_URL`. Il coûte le délai de connexion du premier (`DB_CONNECT_TIMEOUT`, 2 s par défaut).

> **À vérifier dans votre `.env`** : les deux sources peuvent se contredire, et `DATABASE_URL` l'emporte tant qu'elle répond. Un `DB_ENGINE=sqlite` laissé à côté d'une adresse de serveur devient la base utilisée dès que le serveur ne répond plus. En production, ne gardez qu'une source, ou deux qui désignent la même base.

`runique migration up | down | status` cherchent de la même façon l'adresse qui répond, puis la transmettent à la crate `migration`, qui ne lit que `DATABASE_URL` : l'application et les migrations visent toujours la même base. Le mot de passe n'apparaît jamais dans un message d'erreur.

---

## PostgreSQL

### 1. Installer PostgreSQL

**macOS :**

```bash
brew install postgresql
brew services start postgresql
```

**Linux (Debian/Ubuntu) :**

```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows :** [télécharger l'installeur](https://www.postgresql.org/download/windows/) et suivre l'assistant.

### 2. Créer l'utilisateur et la base de données

```sql
-- En tant qu'administrateur : psql -U postgres
CREATE USER runique_user WITH PASSWORD 'secure_password';
CREATE DATABASE mon_projet OWNER runique_user;
GRANT ALL PRIVILEGES ON DATABASE mon_projet TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

### 3. Vérifier la connexion

```bash
psql -U runique_user -d mon_projet -h localhost
```

---

## Limites actuelles

- **Un moteur par compilation.** L'ORM (SeaORM) ne parle qu'à un moteur à la fois (sauf `all-databases`, réservé aux outils). Rien n'empêche d'attacher **en plus** une base externe — Redis, MongoDB, un PostgreSQL de rab : clin d'œil à [`with_custom_db`](/docs/fr/configuration/builder).
- **SQLite n'accepte qu'un écrivain à la fois** sur toute la base : sous forte charge en écriture, une requête peut attendre puis échouer (`SQLITE_BUSY`). C'est le moteur de développement ; préférez PostgreSQL en production.
- **Les migrations générées sont communes à tous les moteurs** : ce qui ne concerne qu'un moteur (type natif `CREATE TYPE` des enums sous PostgreSQL, clés étrangères d'un cycle) est choisi à l'exécution, dans le fichier. Changer de moteur ne demande pas de régénérer les migrations, mais une **base neuve** : les données ne sont pas transférées d'un moteur à l'autre.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Prérequis](/docs/fr/installation/prerequis) | Setup initial |
| [Migrations](/docs/fr/installation/migrations) | Workflow de migration |

## Retour au sommaire

- [Installation](/docs/fr/installation)
