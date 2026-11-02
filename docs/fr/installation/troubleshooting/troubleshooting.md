# Troubleshooting

## `no such table: eihwaz_sessions` (ou `relation "eihwaz_sessions" does not exist`)

Les migrations n'ont pas été appliquées sur cette base.

```bash
runique migration up
```

L'erreur peut sembler intermittente : une page s'affiche sans cookie de session (session anonyme, en mémoire), puis échoue dès que le navigateur renvoie un cookie que le serveur ne connaît plus, par exemple après un redémarrage, et va le chercher en base. En mode debug, la page d'erreur peut alors être vide : la cause est dans les logs du terminal (`ERROR tower_sessions_core::session`).

---

## « driver not enabled » au démarrage

Le moteur demandé par `.env` (`DB_ENGINE` / `DATABASE_URL`) n'est pas celui pour lequel Runique a été compilé. Le message indique le moteur compilé : alignez `.env` dessus, ou changez la feature dans `Cargo.toml` **et** `migration/Cargo.toml`. Voir [Base de données](/docs/fr/installation/base-de-donnees).

Pour la CLI (`create-superuser`), c'est le binaire installé qui doit avoir la feature : `cargo install runique --features <moteur> --locked --force`.

---

## `can't find crate for serde` sur chaque `model!` / `#[form]`

Les dépendances `serde`, `sea-orm`… ne sont pas dans `[dependencies]`. Cause habituelle : une section `[dev-dependencies]` insérée **au milieu** de `Cargo.toml`. En TOML, une ligne appartient à la dernière section ouverte au-dessus d'elle : tout ce qui suit `[dev-dependencies]` devient une dépendance de test seulement.

```toml
[dependencies]
runique = { version = "4.0.0", features = ["sqlite"] }
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
sea-orm = "=2.0.4"
sea-orm-migration = "=2.0.4"

# Toujours en dernier
[dev-dependencies]
runique = { version = "4.0.0", features = ["test-utils"] }
```

Les erreurs « `Column` / `Entity` introuvable » qui suivent n'en sont que la conséquence : le `model!` a échoué, il n'a rien généré.

---

## Déconnecté en passant d'un projet à l'autre en local

Deux projets Runique lancés sur le même hôte (`127.0.0.1:3000` et `127.0.0.1:3001`) partagent le même cookie de session : **les cookies ne tiennent pas compte du port**. Le second projet ne connaît pas la session, en crée une et remplace le cookie : le premier vous voit déconnecté. Ouvrez l'un sur `localhost` et l'autre sur `127.0.0.1`, deux noms d'hôte distincts pour le navigateur.

Ce n'est pas une faille : un identifiant de session n'est valable que pour la base qui l'a créé. Deux projets qui partagent la même base partagent aussi leurs sessions.

---

## `makemigrations` ne génère rien alors que les tables n'existent pas

Les snapshots de `migration/src/snapshots/` indiquent un schéma « déjà migré ». S'ils ont été copiés d'un autre projet sans leurs fichiers de migration, `makemigrations` ne voit aucun changement. Supprimez `migration/src/snapshots/` et `migration/src/lib.rs`, puis relancez `runique makemigrations`.

---

## `Connection refused` (PostgreSQL)

```bash
sudo systemctl status postgresql   # Linux
brew services list                 # macOS
```

---

## `permission denied` sur la base de données (PostgreSQL)

```sql
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

---

## Le port 3000 est déjà utilisé

Changez `PORT` dans `.env`.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Prérequis](/docs/fr/installation/prerequis) | Setup initial |
| [Base de données](/docs/fr/installation/base-de-donnees) | Changer de moteur |
| [Migrations](/docs/fr/installation/migrations) | Workflow et limites |

## Retour au sommaire

- [Installation](/docs/fr/installation)
