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
    },
    articles: articles::Model => ArticleForm {
        title: "Articles",
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

RuniqueApp::builder(config)
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
| `.prefix("/admin")` | Préfixe des routes admin (défaut : `/admin`) |
| `.site_title("…")` | Titre affiché dans l'interface |
| `.auth(RuniqueAdminAuth::new())` | Authentification admin (par défaut) |
| `.routes(admins::routes("/admin"))` | Monte les routes CRUD sous `/admin` |
| `.with_state(…)` | État partagé généré par le daemon |
| `.no_robots_txt()` | Désactive le `/robots.txt` automatique |
| `.extra_routes(vec![…])` | Routes custom protégées par le middleware admin |

> **robots.txt automatique** — Quand l'admin est actif, Runique génère automatiquement
> une route `/robots.txt` contenant `Disallow: /admin/` pour exclure l'interface
> des moteurs de recherche. Le préfixe configuré via `.prefix()` est respecté.
> Utilisez `.no_robots_txt()` si vous souhaitez gérer ce fichier vous-même.

---

## Routes admin custom (`extra_routes`)

Pour des vues métier qui dépassent le CRUD généré (ex : détail commande avec lignes associées,
tableau de bord personnalisé), déclarez des routes supplémentaires dans `url.rs` et
enregistrez-les via `.extra_routes()`.

Ces routes héritent automatiquement du middleware admin : authentification, `AdminState`,
`PrototypeAdminState` (sidebar), et `CurrentUser`.

**`url.rs`**

```rust
pub fn admin_extra_routes() -> Vec<(&'static str, runique::axum::routing::MethodRouter)> {
    vec![
        ("/commandes/{numero}/detail", view!{ admin_commande_detail }),
    ]
}
```

**`main.rs`**

```rust
.with_admin(|a| a
    .routes(admins::routes("/admin"))
    .extra_routes(url::admin_extra_routes())
)
```

Les chemins sont **relatifs au préfixe admin** — `/commandes/{numero}/detail` devient
`/admin/commandes/{numero}/detail`.

**Handler** — récupérer les extensions injectées par le middleware :

```rust
use runique::prelude::*;
use runique::admin::{AdminState, PrototypeAdminState, inject_admin_prefix, insert_admin_messages};

pub async fn admin_commande_detail(
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(proto): Extension<Arc<PrototypeAdminState>>,
    mut request: Request,
) -> AppResult<Response> {
    inject_admin_prefix(&mut request, &admin.config.prefix);
    insert_admin_messages(&mut request, proto.registry.all());
    // … logique métier …
    request.render("admin/commande_detail.html")
}
```

Le template doit étendre `admin_base` pour hériter du CSS et de la sidebar :

```html
{% extends "admin_base" %}
```

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
| [CLI](/docs/fr/admin/declaration) | Commande `runique start`, workflow général |
| [Permissions](/docs/fr/admin/permission) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état bêta |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
