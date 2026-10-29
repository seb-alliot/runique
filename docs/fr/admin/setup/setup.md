# Mise en place de l'admin

Guide pas à pas pour activer l'interface d'administration avec le builder `ModelAdmin`.

## Prérequis

- Un projet Runique avec base de données configurée
- Des modèles déclarés via `model!` (entités SeaORM générées)
- Un compte staff ou superuser pour se connecter

## Étape 1 — Créer `src/admin.rs`

Une fonction de config reçoit `AdminStaging` et enregistre les modèles :

```rust
// src/admin.rs
use crate::entities::{article, user_profile};
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;

pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .page_size(20)
        .site(site)
}

fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())
        .register(articles())
}

fn accounts() -> ModelAdmin<user_profile::Entity> {
    use user_profile::Column::*;
    ModelAdmin::new()
        .list(|l| {
            l.display([Username, Email, IsActive])
                .sortable([Username])
                .search([Username, Email])
        })
        .form(|f| f.fields([Username, Email]))
}

fn articles() -> ModelAdmin<article::Entity> {
    use article::Column::*;
    ModelAdmin::new()
        .title("Articles")
        .list(|l| l.display([Title, Status]).links([Title]).filter([Status]))
        .form(|f| f.fields([Title, Body, Status]))
}
```

Points importants :

- `accounts_users(...)` remplace l'admin comptes du framework (table étendue via `extend!`)
- `register(...)` ajoute un modèle métier
- Comptes et groupes existent déjà côté framework ; tu n'as pas à les recréer

## Étape 2 — Déclarer le module

```rust
// src/main.rs
mod admin;
```

Plus de dossier `src/admins/` généré : tout vit dans ton `admin.rs`.

## Étape 3 — Câbler `.with_admin()`

```rust
use runique::app::builder::RuniqueAppBuilder as builder;

builder::new(config)
    .routes(url::routes())
    .with_database(db)
    .with_admin(admin::config)
    .build()
    .await?
    .run()
    .await?;
```

Tu peux aussi passer une closure inline :

```rust
.with_admin(|a| {
    a.site_title("Admin")
        .path("/admin/")
        .site(|s| s.register(ModelAdmin::<article::Entity>::new()))
})
```

## Path et prefix

Deux réglages distincts :

| Méthode | Rôle |
| --- | --- |
| `.path("/admin/")` | Où vit l'admin (segment CRUD) |
| `.prefix("secret")` | Segment monté **devant** le path |

| `path` | `prefix` | URL finale |
| --- | --- | --- |
| `/admin/` | — | `/admin/…` |
| `/admin/` | `secret` | `/secret/admin/…` |
| `/admin-runique/` | `prefix-test` | `/prefix-test/admin-runique/…` |

Exemple demo-app :

```rust
admin
    .path("/admin-runique/")
    .prefix("/prefix-test")
```

## Options utiles de `AdminStaging`

| Méthode | Effet |
| --- | --- |
| `.site_title("…")` | Titre dans l'UI |
| `.page_size(15)` | Taille de page liste par défaut |
| `.search_max_len(n)` | Longueur max du terme de recherche |
| `.with_rate_limiter(...)` | Rate limit login / actions |
| `.with_login_guard(...)` | Verrouillage après N échecs |
| `.templates(|t| …)` | Surcharge globale des templates |
| `.sitemap("https://…")` | Directive Sitemap dans `/robots.txt` |
| `.no_robots_txt()` | Désactive le robots.txt auto |
| `.disable()` / `.enable()` | Coupe ou active le panel |

## Créer un superuser

```bash
runique createsuperuser
```

Le compte obtenu a `is_superuser` (et le staff nécessaire) pour ouvrir le panel.

## Vérifier au boot

La config est validée au démarrage. En demo-app, un test unitaire rejoue les mêmes contrôles :

```rust
#[test]
fn every_registered_model_passes_the_boot_checks() {
    let staging = super::config(AdminStaging::new().enable());
    if let Err(e) = staging.validate() {
        panic!("{e}");
    }
}
```

## Voir aussi

| Section | Description |
| --- | --- |
| [Déclaration](/docs/fr/admin/declaration) | `AdminSite` et enregistrement |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | API complète liste / form |
| [Permissions](/docs/fr/admin/permission) | Qui voit quoi |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
