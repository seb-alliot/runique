# Admin Runique

L'administration Runique construit un CRUD complet à partir d'un **builder typé** (`ModelAdmin`). Plus de macro `admin!`, plus de génération de fichiers : tu déclares des colonnes SeaORM, le framework sert les routes, la liste, les formulaires et les permissions.

Les colonnes sont des enums (`blog::Column::Title`) — le compilateur refuse une colonne inexistante, un secret exposé ou une action incohérente.

## Exemple minimal

```rust
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use crate::entities::article;

pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .site(|site| site.register(articles()))
}

fn articles() -> ModelAdmin<article::Entity> {
    use article::Column::*;
    ModelAdmin::new()
        .title("Articles")
        .list(|l| {
            l.display([Title, Status, Views])
                .links([Title])
                .sortable([Title])
                .search([Title])
                .filter([Status])
        })
        .form(|f| f.fields([Title, Body, Status]))
}
```

Puis dans `main` :

```rust
builder::new(config)
    .routes(url::routes())
    .with_database(db)
    .with_admin(admin::config)
    .build()
    .await?
    .run()
    .await?;
```

## Table des matières

| Fichier | Contenu |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin, path, prefix, superuser |
| [Déclaration](/docs/fr/admin/declaration) | `AdminStaging`, `AdminSite`, enregistrement |
| [Builder `ModelAdmin`](/docs/fr/admin/declaration-macro) | API liste + formulaire, colonnes typées |
| [Validation au boot](/docs/fr/admin/declaration-daemon) | Contrôles exécutés au démarrage |
| [Création d'utilisateur](/docs/fr/admin/user-creation) | Flux création → email → activation |
| [Vue liste](/docs/fr/admin/liste) | Pagination, tri, recherche, filtres, actions |
| [Permissions](/docs/fr/admin/permission) | Rangs, groupes, grille de droits |
| [Templates](/docs/fr/admin/template) | Surcharge des pages et des blocs |
| [Évolutions](/docs/fr/admin/evolution) | Pistes futures |

## Revenir au menu

- [Readme Francais](https://github.com/seb-alliot/runique/blob/main/README.fr.md)
