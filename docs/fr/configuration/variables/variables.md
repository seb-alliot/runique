# Fichier `.env`

La configuration de l'application se lit dans le fichier `.env` à la racine du projet, puis se complète dans le code (builder). La liste complète des variables, avec leurs valeurs par défaut, est dans [Variables d'environnement](/docs/fr/env).

`runique new` écrit un `.env` prêt à l'emploi, en SQLite, avec un `SECRET_KEY` aléatoire propre au projet.

---

## Exemples complets

`RUNIQUE_ENV` règle le comportement de l'application ; `DEBUG` n'affiche que les pages d'erreur détaillées. Voir [Application & Serveur](/docs/fr/env/application).

### Développement (SQLite)

```env
IP_SERVER=127.0.0.1
PORT=3000
RUNIQUE_ENV=dev
DEBUG=true

SECRET_KEY=<généré par runique new>

DB_ENGINE=sqlite
DATABASE_URL="sqlite://data.db?mode=rwc"
```

`?mode=rwc` crée le fichier s'il n'existe pas. L'URL doit commencer par `sqlite://` : une forme courte comme `sqlite:data.db` est refusée (« Unsupported database URL scheme »).

### Développement (PostgreSQL)

```env
IP_SERVER=127.0.0.1
PORT=3000
RUNIQUE_ENV=dev
DEBUG=true

SECRET_KEY=<généré par runique new>

DB_ENGINE=postgres
DATABASE_URL=postgres://runique_user:password@localhost:5432/mon_projet
```

Ou, au lieu de `DATABASE_URL`, les variables composantes :

```env
DB_ENGINE=postgres
DB_USER=runique_user
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mon_projet
```

Quand `DATABASE_URL` est définie, elle l'emporte : les variables `DB_*` sont ignorées.

### Production (derrière un reverse proxy)

```env
IP_SERVER=127.0.0.1
PORT=3000
RUNIQUE_ENV=prod
DEBUG=false

SECRET_KEY=<une clé propre à ce déploiement>

DB_ENGINE=postgres
DATABASE_URL=postgres://runique_user:password@db.interne:5432/mon_projet

ENFORCE_HTTPS=true
```

Le proxy (Nginx, Caddy…) écoute sur 80/443 et gère TLS ; Runique reste sur un port interne. Pour que Runique gère lui-même TLS sans proxy, voir [Réseau & HTTP](/docs/fr/installation/network) (ACME).

En production, Runique refuse de démarrer si `SECRET_KEY` est faible, ou si la réinitialisation de mot de passe ou l'admin sont activés sans `.with_public_url(...)`.

---

## Générer une clé secrète

`runique new` en génère une à la création du projet. Pour en créer une autre (fuite, nouveau déploiement) : n'importe quelle chaîne aléatoire d'au moins 32 caractères.

```bash
openssl rand -hex 32
```

---

## Ce qui ne se règle pas dans `.env`

Les hosts autorisés, la CSP, CORS, les sessions et les middlewares se configurent dans le builder (`main.rs`) :

```rust
.middleware(|m| m.with_allowed_hosts(|h| h.host("exemple.com")))
```

Voir [Builder](/docs/fr/configuration/builder).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Variables d'environnement](/docs/fr/env) | Toutes les variables et leurs valeurs par défaut |
| [Accès dans le code](/docs/fr/configuration/code) | `RuniqueConfig` |
| [Builder](/docs/fr/configuration/builder) | Configuration programmatique |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
