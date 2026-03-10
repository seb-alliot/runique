# Mise en place de l'admin

Guide pas à pas pour activer l'interface d'administration dans un projet Runique existant.

---

## Prérequis

- Un projet Runique fonctionnel avec une base de données configurée
- Un modèle `users` avec les champs `is_staff` et `is_superuser` (générés par `model!`)
- Le binaire `runique` installé (`cargo install runique` ou `cargo build` du workspace)

---

## Étape 1 — Créer `src/admin.rs`

Ce fichier déclare les ressources administrables via la macro `admin!` :

```rust
// src/admin.rs
use crate::entities::{users, articles};
use crate::formulaire::{RegisterForm, ArticleForm};

admin! {
    users: users::Model => RegisterForm {
        title: "Utilisateurs",
        permissions: ["admin"]
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
        permissions: ["admin"]
    }
}
```

---

## Étape 2 — Générer `src/admins/` avec le daemon

```bash
runique start
```

Le daemon lit `src/admin.rs`, génère `src/admins/` et lance `cargo run`.
Le dossier `src/admins/` est créé automatiquement — ne pas le modifier manuellement.

```text
src/admins/
  ├── README.md
  ├── mod.rs
  └── admin_panel.rs
```

> Si `src/admins/` existe déjà depuis une génération précédente, `runique start` le régénère.

---

## Étape 3 — Déclarer le module dans `src/main.rs`

```rust
mod admin;
mod admins;  // module généré par runique start
```

---

## Étape 4 — Câbler `.with_admin()` dans le builder

```rust
use runique::app::builder::RuniqueAppBuilder as builder;

builder::new(config)
    .routes(url::routes())
    .with_database(db)
    .with_admin(|a| {
        a.site_title("Administration")
         .auth(RuniqueAdminAuth::new())
         .routes(admins::routes("/admin"))
         .with_state(admins::admin_state())
    })
    .build()
    .await?
    .run()
    .await?;
```

| Méthode | Rôle |
| --- | --- |
| `.site_title("…")` | Titre affiché dans l'interface |
| `.auth(RuniqueAdminAuth::new())` | Authentification admin (par défaut) |
| `.routes(admins::routes("/admin"))` | Monte les routes CRUD sous `/admin` |
| `.with_state(…)` | État partagé généré par le daemon |

---

## Étape 5 — Créer un superuser

```bash
runique create-superuser
```

Suit un assistant interactif pour créer le premier compte admin (`is_superuser = true`).

---

## Accès à l'interface

Une fois le serveur démarré, l'interface est disponible à :

```text
http://localhost:{PORT}/admin/
```

La page `/admin/login` redirige vers le dashboard si les identifiants sont valides.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/declaration/cli.md) | Commande `runique start`, workflow général |
| [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/permission/permissions.md) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/template/templates.md) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/evolution/evolution.md) | Axes d'évolution et état bêta |

## Revenir au sommaire

- [Sommaire Admin](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/11-Admin.md)
