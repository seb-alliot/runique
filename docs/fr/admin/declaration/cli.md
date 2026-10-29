# Déclarer l'admin

L'admin se déclare en Rust pur : un `AdminStaging` pour le site, un `AdminSite` pour les modèles, un `ModelAdmin<E>` par entité.

## Flux

```text
main.rs
  .with_admin(admin::config)
       │
       ▼
admin.rs :: config(AdminStaging) -> AdminStaging
  .site_title / .path / .page_size / …
  .site(|site| site.register(…).accounts_users(…))
       │
       ▼
Au build : validate() puis montage des routes CRUD
```

Aucune génération de code, aucun daemon, aucun `src/admins/`.

## `AdminStaging` — le site

Reçu par la closure `.with_admin(...)`. Méthodes principales :

```rust
pub fn config(admin: AdminStaging) -> AdminStaging {
    admin
        .site_title("Administration")
        .path("/admin/")
        .prefix("backoffice")   // optionnel
        .page_size(20)
        .with_rate_limiter(RateLimiter::new().max_requests(20).retry_after(3600))
        .with_login_guard(LoginGuard::new().max_attempts(20).lockout_secs(300))
        .templates(|t| t.with_dashboard("admin/mon_dashboard.html"))
        .site(site)
}
```

## `AdminSite` — les modèles

```rust
pub fn site(site: AdminSite) -> AdminSite {
    site.accounts_users(accounts())  // remplace l'admin comptes framework
        .register(blog())
        .register(article())
        .register(ModelAdmin::<tag::Entity>::new()) // défaut OK
}
```

| Méthode | Rôle |
| --- | --- |
| `.register(ModelAdmin<E>)` | Ajoute un modèle métier |
| `.accounts_users(ModelAdmin<E>)` | Remplace l'admin des comptes (extension `eihwaz_users`) |

Comptes et groupes sont toujours présents. Enregistrer un second modèle sous la clé comptes est refusé au boot.

## `ModelAdmin<E>` — un modèle

```rust
fn blog() -> ModelAdmin<blog::Entity> {
    use blog::Column::*;
    ModelAdmin::new()
        .title("Blog")
        .list(|l| { /* ListConfig */ })
        .form(|f| { /* FormConfig */ })
        .templates(|t| { /* surcharge pages de ce modèle */ })
}
```

Sans `.list().display(...)`, le framework affiche toutes les colonnes non secrètes (sur les comptes : username, email, is_active).

Sans `.form().fields(...)`, le formulaire expose tous les champs du modèle (hors secrets et drapeaux de rang).

## Imports utiles

```rust
use runique::admin::{AdminSite, ModelAdmin};
use runique::app::staging::AdminStaging;
use sea_orm::Order;
```

## Voir aussi

| Section | Description |
| --- | --- |
| [Builder ModelAdmin](/docs/fr/admin/declaration-macro) | Détail de l'API |
| [Validation au boot](/docs/fr/admin/declaration-daemon) | Ce qui est contrôlé au démarrage |
| [Mise en place](/docs/fr/admin/setup) | Câblage dans main |

## Revenir au sommaire

- [Sommaire Admin](/docs/fr/admin)
