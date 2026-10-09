🌍 **Langues** : [English](TESTING.md) | [Français](TESTING.fr.md)

# Tester Runique et lancer demo-app en local

Le dépôt contient deux choses distinctes :

| Quoi | Base de données | Docker |
| --- | --- | --- |
| La suite de tests (`runique/`) | SQLite en mémoire, plus Postgres et MariaDB quand ils sont disponibles | Facultatif |
| Le site de démonstration (`demo-app/`, runique.io) | **Postgres uniquement** | Recommandé |

---

## Prérequis

| Outil | Pour | Installation |
| --- | --- | --- |
| Rust 1.94+ | Les tests, demo-app sur la machine | [rustup](https://rustup.rs) |
| Docker | Postgres et MariaDB (facultatif pour les tests), ou demo-app en entier | [docs.docker.com](https://docs.docker.com/get-docker/) |
| `psql` | demo-app sur la machine : le seed charge `seed.sql` (pages, exemples de code) par cet outil | paquet `postgresql-client` (`apt install postgresql-client`) |
| `sea-orm-cli` | demo-app sur la machine : `runique migration up` délègue à cet outil | `cargo install sea-orm-cli` |
| CLI `runique` | demo-app sur la machine : migrations et compte administrateur | `cargo install --path runique --features postgres` |

La CLI `runique` s'installe depuis le workspace, pour suivre la version du framework : lancez
la commande à la racine du dépôt, là où `--path runique` désigne le dossier du framework. La feature
`postgres` est nécessaire : compilée seule, la CLI n'active aucun pilote de base. Réinstallez-la
après chaque mise à jour du workspace.

---

## 1. Lancer la suite de tests

### SQLite seul — rien à installer

Chaque test qui a besoin d'une base en reçoit une neuve, en SQLite en mémoire. Les tests
Postgres et MariaDB sont ignorés, pas en échec, quand aucune URL n'est configurée.

```bash
cd runique
cargo test --features orm,sqlite,postgres,mysql,mariadb,all-databases,acme
```

C'est exactement le jeu de features utilisé par la CI.

### Avec Postgres et MariaDB

Le `docker-compose.yml` de la racine démarre les deux moteurs pour les tests :

```bash
docker compose up -d
```

Créez ensuite `runique/.env.test` :

```env
DATABASE_URL_PG=postgres://runique:runique_test@localhost:5433/runique_test
DATABASE_URL_MARIADB=mysql://runique:runique_test@localhost:3307/runique_test
```

Chaque clone du dépôt, et chaque variante de clé primaire (voir plus bas), reçoit sa propre
base, créée à la première utilisation : deux copies, ou une passe `i32` et une passe `pk-uuid`,
n'écrasent pas les tables l'une de l'autre. Ne lancez pas deux fois **la même** variante en même
temps depuis la même copie : les deux passes partageraient cette base.

### Variantes de clé primaire

La CI lance la suite trois fois, une par type de clé primaire :

```bash
cd runique
cargo test --features big-pk,all-databases
cargo test --features pk-uuid,all-databases
```

`cargo clippy --features big-pk` (ou `pk-uuid`) seul n'active aucun moteur de base : définissez
`DB_ENGINE=postgres` pour lui, comme le fait la CI, sinon les macros de modèle ne peuvent pas
choisir de moteur.

---

## 2. Lancer demo-app en local

demo-app ne tourne que sur Postgres : son `Cargo.toml` active la feature `postgres`, et son
`seed.sql`, rejoué à chaque démarrage, utilise des types Postgres (`CREATE TYPE … AS ENUM`,
séquences).

### Tout dans Docker

Seul Docker est nécessaire : ni Rust, ni les CLI. Depuis la racine du dépôt :

```bash
docker compose --profile demo up -d --build
```

Le premier build compile le workspace et prend plusieurs minutes. Le conteneur applique les
migrations à chaque démarrage, puis lance le site sur `http://127.0.0.1:3000`. Les clés
facultatives (tableau plus bas) se placent dans `demo-app/.env` : le conteneur le lit s'il
existe, mais garde sa propre base et sa propre `DATABASE_URL`.

Compte administrateur, avec la CLI déjà présente dans l'image :

```bash
docker compose exec demo runique create-superuser
```

Les étapes de l'assistant et les règles du mot de passe : [3. Créer le compte administrateur](#3-créer-le-compte-administrateur).

Après une modification du code : `docker compose --profile demo up -d --build demo`.

### Sur la machine

**1. Démarrer Postgres**

```bash
docker compose up -d postgres
```

La base `runique_demo` est créée au premier démarrage du conteneur
(`docker/postgres-init/01-demo.sql`). Un conteneur Postgres créé avant ce script n'a pas cette
base : créez-la avec `docker compose exec postgres createdb -U runique runique_demo`.

Un Postgres installé sur la machine convient aussi : faites pointer `DATABASE_URL` dessus.

**2. Créer `demo-app/.env`**

```env
DEBUG=true
DB_ENGINE=postgres
DATABASE_URL=postgres://runique:runique_test@localhost:5433/runique_demo
SECRET_KEY=a-changer-avec-au-moins-32-caracteres-aleatoires
```

Clés facultatives :

| Clé | Pour |
| --- | --- |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASS`, `SMTP_FROM`, `SMTP_STARTTLS` | L'envoi d'emails (activation de compte, réinitialisation du mot de passe). Sans elles, l'admin affiche le lien de réinitialisation à l'écran |
| `GROQ_API_KEY` | Le retour de l'IA sur les exercices des cours (`/cours/…/exercice`) |
| `RUNIQUE_MAX_UPLOAD_MB` | Les uploads de plus de 2 Mo |

**3. Appliquer les migrations**

```bash
cd demo-app
runique migration up
```

Elles créent les tables du framework (comptes, sessions, groupes) et celles de la démo.

**4. Lancer**

```bash
cargo run -p demo-app
```

Le site répond sur `http://127.0.0.1:3000`. Le contenu (docs, cours, exemples) est chargé depuis
`seed.sql` à chaque démarrage : le modifier ne demande aucune migration.

**5. Accès à l'admin**

Le site occupe le premier terminal : ouvrez-en un second, à la racine du dépôt. La CLI tourne
sur votre machine, pas dans Docker : elle lit `DATABASE_URL` dans `demo-app/.env` et rejoint le
conteneur Postgres par le port 5433. Il doit donc être démarré (`docker compose ps`).

```bash
cd demo-app
runique create-superuser
```

Les étapes de l'assistant et les règles du mot de passe : [3. Créer le compte administrateur](#3-créer-le-compte-administrateur).

`runique start` régénère `src/admins/` à partir de `src/admin.rs` avant de lancer le site :
nécessaire seulement après avoir modifié les déclarations `admin!{}`.

---

## 3. Créer le compte administrateur

`runique create-superuser` est un assistant interactif : lancez-le dans un vrai terminal (sous
Docker, par `docker compose exec`, qui en fournit un ; l'option `-T` l'empêcherait de
fonctionner).

| Étape | Que saisir |
| --- | --- |
| 1. Algorithme | Gardez **Argon2**, le choix par défaut (Entrée). Bcrypt, Scrypt ou un programme externe sont les alternatives |
| 2. Nom d'utilisateur | Un nom pas encore pris |
| 3. Email | Un email valide, pas encore pris ; enregistré en minuscules |
| 4. Mot de passe | Au moins **12 caractères**, avec une minuscule, une majuscule, un chiffre **et un caractère spécial** (`-`, `!`, `@`…) ; saisi deux fois, jamais affiché |
| 5. Récapitulatif | Confirmez, ou revenez modifier une étape |

Le compte est créé actif, membre du staff et superutilisateur. Connectez-vous sur
`http://127.0.0.1:3000/prefix-test/admin-runique/`.

Ctrl+C quitte sans rien créer.
