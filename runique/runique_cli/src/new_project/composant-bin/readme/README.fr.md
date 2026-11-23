# {{PROJECT_NAME}}

Une application web construite avec [Runique](https://runique.io/docs/fr),
générée par `runique new` (Runique v{{RUNIQUE_VERSION}}).

## Démarrage rapide

```bash
# 1. Créer les tables du framework (comptes, sessions, droits)
runique migration up

# 2. Lancer l'application
cargo run
```

Puis ouvre **http://127.0.0.1:3000**. Le projet fournit l'inscription, la
connexion et la déconnexion.

Le premier `runique migration up` compile le crate `migration`, il prend donc un
peu de temps ; les suivants sont rapides.

## Tes propres modèles

Déclare chaque modèle dans son propre fichier sous `src/entities/`, avec `model!` :

```rust
// src/entities/article.rs
use runique::prelude::*;

model! {
    Article,
    table: "articles",
    pk: id => Pk,
    {
        title:      text     [required, max_length: 150],
        body:       textarea [required],
        created_at: datetime [auto_now],
    }
}
```

Déclare-le dans `src/entities/mod.rs` (`pub mod article;`), puis :

```bash
runique makemigrations   # compare les modèles à leurs snapshots, écrit la migration
runique migration up     # l'applique
```

Pour modifier un modèle plus tard, change-le et relance les deux mêmes commandes :
`makemigrations` écrit une migration `ALTER`. Un changement destructif (colonne
supprimée, longueur réduite, `nullable` devenu obligatoire…) est bloqué, sauf
avec `--force`.

> **Changer le type d'une colonne n'est pas encore pris en charge.**
> `makemigrations` le bloque ; avec `--force`, la migration écrite ne contient
> qu'un commentaire d'avertissement, aucun `ALTER`, alors que le snapshot
> enregistre déjà le nouveau type. La migration « s'applique » donc sans toucher
> à la base, et rien ne le signale ensuite. En attendant : écris à la main le
> `up()` et le `down()` de cette migration (`alter_table` / `modify_column` de
> SeaORM) avant `migration up`, ou ajoute une nouvelle colonne, recopie les
> données, puis supprime l'ancienne.

## Migrations

| Commande | Ce qu'elle fait |
|---|---|
| `runique makemigrations` | Écrit les migrations des modèles qui ont changé |
| `runique makemigrations --force` | Pareil, en autorisant les changements destructifs |
| `runique migration up` | Applique toutes les migrations en attente |
| `runique migration down -n 1` | Annule la dernière migration (son `down()` s'exécute) |
| `runique migration status` | Liste les migrations et indique lesquelles sont appliquées |

`migration/` est un crate de migrations SeaORM standard : `makemigrations` tient
son `src/lib.rs` à jour, il n'y a rien à y écrire à la main. Les commandes
`runique migration` le lancent pour toi et lisent `DATABASE_URL` dans `.env`.

## Structure du projet

```
{{PROJECT_NAME}}/
├── migration/          le crate de migrations (Cargo.toml, src/main.rs, src/lib.rs)
├── src/
│   ├── entities/       tes modèles (model!)
│   ├── forms/          formulaires d'inscription et de connexion
│   ├── main.rs         le builder de l'application
│   ├── url.rs          les routes
│   └── views.rs        les vues
├── templates/          templates Tera (base.html est la mise en page)
├── static/             CSS
├── media/              favicon, logo, et les fichiers envoyés par les utilisateurs
├── .env                réglages et SECRET_KEY (ne jamais le commiter)
└── Cargo.toml
```

## Versions

Runique v{{RUNIQUE_VERSION}} est construit avec SeaORM `{{SEA_ORM_REQ}}` : `Cargo.toml` et
`migration/Cargo.toml` imposent cette version exacte. Ne la change pas à la main ;
quand tu mets Runique à jour, aligne les deux fichiers sur la version de la
nouvelle release.

## Configuration

`.env` contient les réglages : `IP_SERVER` et `PORT` (où le serveur écoute),
`DEBUG`, `SECRET_KEY` (générée pour ce projet : garde-la secrète, et mets-en une
nouvelle à chaque déploiement) et la base de données (`DB_ENGINE`, `DATABASE_URL`).

Pour passer de SQLite à PostgreSQL ou MySQL, change les trois ensemble : la
feature `runique` dans `Cargo.toml` et dans `migration/Cargo.toml` (avec la
feature `sqlx-postgres` / `sqlx-mysql` correspondante de `sea-orm-migration`),
et `DB_ENGINE` / `DATABASE_URL` dans `.env`.

## Aller plus loin

- Réinitialisation du mot de passe par email : `.with_password_reset(...)` sur le builder, avec un mailer.
- Panneau d'administration : la feature `admin` de `runique`, puis `.with_admin(...)`.

Les deux sont décrits dans la documentation : **https://runique.io/docs/fr**

## Dépannage

- **« no such table »** : les migrations ne sont pas encore appliquées. Lance `runique migration up`.
- **Port 3000 déjà utilisé** : change `PORT` dans `.env`.
- **La base de données est introuvable** : lance `cargo run` et `runique migration`
  depuis la racine du projet, là où se trouve `.env` ; un chemin SQLite relatif
  est lu depuis là.
