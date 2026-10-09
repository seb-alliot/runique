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
| Rust 1.94+ | Tout | [rustup](https://rustup.rs) |
| Docker | Postgres et MariaDB (facultatif pour les tests, recommandé pour demo-app) | [docs.docker.com](https://docs.docker.com/get-docker/) |
| `sea-orm-cli` | demo-app : `runique migration up` délègue à cet outil | `cargo install sea-orm-cli` |
| CLI `runique` | demo-app : migrations et compte administrateur | `cargo install --path runique --features postgres` |

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

Chaque clone du dépôt reçoit sa propre base, créée à la première utilisation (nommée d'après
le chemin du clone) : deux copies du dépôt n'écrasent pas les tables l'une de l'autre. Ne
lancez pas deux `cargo test` en même temps depuis la **même** copie : ils partageraient ces bases.

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

**1. Démarrer Postgres et créer la base de la démo**

```bash
docker compose up -d postgres
docker compose exec postgres createdb -U runique runique_demo
```

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

L'admin est sur `http://127.0.0.1:3000/prefix-test/admin-runique/`.

`runique start` régénère `src/admins/` à partir de `src/admin.rs` avant de lancer le site :
nécessaire seulement après avoir modifié les déclarations `admin!{}`.
