# Prérequis & premier projet

## Prérequis

- **Rust 1.94+** — [installer rustup](https://rustup.rs/)
- Une base de données : **SQLite** (rien à installer, utilisé par défaut), **PostgreSQL 12+**, ou **MySQL / MariaDB**

```bash
rustc --version    # 1.94 ou plus
```

---

## 1. Installer la CLI Runique

```bash
cargo install runique --features sqlite --locked
```

La CLI se connecte à la base (`create-superuser`) : elle doit être compilée avec le pilote de votre moteur. Remplacez `sqlite` par `postgres`, `mysql` ou `mariadb`, ou utilisez `all-databases` pour un binaire qui parle aux trois. Sans feature de moteur, toute connexion échoue avec « driver not enabled ».

`--locked` utilise les versions de dépendances publiées avec la crate ; sans lui, Cargo peut choisir des versions plus récentes, incompatibles entre elles.

Pour changer de moteur plus tard, réinstallez avec `--force` :

```bash
cargo install runique --features postgres --locked --force
```

---

## 2. Créer le projet

```bash
runique new mon_projet
cd mon_projet
```

Le nom ne peut contenir que des lettres, chiffres, `_` et `-`, ne commence ni par un chiffre ni par `-`, et ne peut pas être un nom réservé (mot-clé Rust, `std`, `core`, `test`, noms de périphériques Windows comme `con`, ou `migration` / `runique`, déjà pris dans le projet généré).

`runique new` génère un projet complet, prêt à lancer :

```
mon_projet/
├── Cargo.toml          workspace : l'application + son crate `migration`
├── .env                réglages, avec un SECRET_KEY aléatoire propre au projet
├── migration/          le crate de migrations (Cargo.toml, src/main.rs, src/lib.rs)
├── src/
│   ├── main.rs         le builder de l'application
│   ├── url.rs          les routes (/, /signup, /login, /logout, /about)
│   ├── views.rs        les vues
│   ├── forms/          formulaires d'inscription et de connexion
│   └── entities/       vos modèles (model!)
├── templates/          base.html (mise en page), index, about, signup, login
├── static/css/
└── media/              favicon, logo, et les fichiers envoyés par les utilisateurs
```

Le projet utilise **SQLite** par défaut. Pour un autre moteur, voir [Base de données](/docs/fr/installation/base-de-donnees).

---

## 3. Créer les tables du framework

```bash
runique migration up
```

Le crate `migration/` contient déjà les migrations des tables du framework (`eihwaz_users`, sessions, droits, historique, jetons de réinitialisation). Cette commande les applique. Le premier lancement compile le crate : il prend un peu de temps, les suivants sont rapides.

> Sans cette étape, l'application démarre mais la première requête qui touche une session échoue avec `no such table: eihwaz_sessions`.

---

## 4. Lancer le serveur

```bash
cargo run
```

**Sortie attendue :**

```
   Runique est opérationnel
      └──>  Serveur lancé sur http://127.0.0.1:3000
          └──>  Connecté à la base de données Postgres -> runique
              └──> ctrl + c pour arrêter
```

La page d'accueil fournit l'inscription, la connexion et la déconnexion.

---

## 5. Créer un superutilisateur (optionnel)

```bash
runique create-superuser
```

Voir [CLI Runique](/docs/fr/installation/cli). Le panneau d'administration demande en plus la feature `admin` : voir [Admin](/docs/fr/admin).

---

## Le fichier `.env`

`runique new` l'écrit pour vous :

```env
IP_SERVER=127.0.0.1
PORT=3000

DEBUG=true

# Signe les jetons CSRF et les sessions : propre à ce projet, à garder secret
SECRET_KEY=<64 caractères hexadécimaux générés par runique new>

DB_ENGINE=sqlite
DB_NAME=data.db
DATABASE_URL="sqlite://data.db?mode=rwc"
```

Le `SECRET_KEY` est généré aléatoirement à la création. Si vous devez le remplacer (fuite, nouveau déploiement), n'importe quelle chaîne aléatoire d'au moins 32 caractères convient, par exemple :

```bash
openssl rand -hex 32
```

En production (`DEBUG=false`), Runique refuse de démarrer avec une clé absente, trop courte ou laissée à sa valeur d'exemple.

Les autres variables (dossiers des templates, des fichiers statiques et des médias, journalisation…) ont des valeurs par défaut : voir [Variables d'environnement](/docs/fr/env).

---

## Outils recommandés (développement)

```bash
rustup component add rust-analyzer clippy rustfmt
```

`sea-orm-cli` n'est **pas** nécessaire : `runique migration up | down | status` lance `cargo run` sur le crate `migration` du projet, dont le `main` appelle la CLI de la bibliothèque `sea-orm-migration`. Voir [Le `Cargo.toml` attendu](/docs/fr/installation/migrations).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](/docs/fr/installation/base-de-donnees) | Changer de moteur |
| [Migrations](/docs/fr/installation/migrations) | Workflow de migration |
| [CLI Runique](/docs/fr/installation/cli) | Toutes les commandes |

## Retour au sommaire

- [Installation](/docs/fr/installation)
