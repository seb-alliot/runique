# Mise en place de l'admin

Guide pas à pas pour activer l'interface d'administration avec le builder `ModelAdmin`.

## Prérequis

- Un projet Runique avec base de données configurée et migrations appliquées (`runique migration up`)
- La feature `admin` activée dans `Cargo.toml` :

  ```toml
  runique = { version = "4.0.0", features = ["postgres", "admin"] }
  ```

- Des modèles déclarés via `model!` (entités SeaORM générées)

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

- `accounts_users(...)` remplace l'admin des comptes du framework par le vôtre, construit sur votre `extend!` de `eihwaz_users`. C'est la seule façon de le remplacer : enregistrer un modèle sous la clé des comptes refuse le démarrage.
- `register(...)` ajoute un modèle métier. **Enregistrez tous vos `model!{}`** : au démarrage, les droits de groupe qui visent un modèle non enregistré sont supprimés de la base.
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

En production (`RUNIQUE_ENV=prod`), ajoutez `.with_public_url("https://mon-site.fr")` : sans elle, le démarrage est refusé dès que l'admin est actif. Les liens envoyés par mail (création de compte, réinitialisation) sont construits sur cette adresse, jamais sur l'en-tête `Host` de la requête, qu'un attaquant peut forger. Avec `RUNIQUE_ENV=dev`, c'est toujours `http://127.0.0.1:{PORT}`.

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
| `.filter_size(8)` | Valeurs par page des groupes de filtre, par défaut |
| `.search_max_len(n)` | Longueur max du terme de recherche |
| `.with_rate_limiter(...)` | Rate limit login / actions |
| `.with_login_guard(...)` | Verrouillage après N échecs |
| `.templates(…)` | Surcharge des templates, voir [Templates](/docs/fr/admin/template-surcharge) |
| `.resource_order([…])` | Ordre des modèles dans le menu (clés non listées à la fin) |
| `.view_site_url("…")` | Lien « retour au site » quand le site public est servi ailleurs (par défaut : `with_public_url`, sinon `/`) |
| `.sitemap("https://…")` | Directive Sitemap dans `/robots.txt` |
| `.no_robots_txt()` | Désactive le robots.txt auto |
| `.disable()` / `.enable()` | Coupe ou active le panel |

## Créer un superuser

```bash
runique create-superuser
```

La commande demande l'algorithme de hachage, le nom, l'email et le mot de passe, puis crée un compte actif avec `is_superuser`. Le superuser a tous les droits ; nommez ensuite un dirigeant depuis la fiche d'un compte (formulaire de rang) pour l'usage courant. Voir [Permissions](/docs/fr/admin/permission).

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
